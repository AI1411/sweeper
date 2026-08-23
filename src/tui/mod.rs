mod app;
pub mod ui;

pub use app::App;

use std::io::{self, stdout};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;

use crate::history::{append_entry, HistoryEntry, KillSignal};
use crate::process::kill::{kill_pid, KillOutcome};
use crate::process::list::list_processes;
use crate::process::ports::listening_ports_cached;
use crate::process::tree::collect_tree_pids;

enum Msg {
    Ports(Vec<(u16, u32)>),
}

pub fn run() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let processes = list_processes();
    let mut app = App::new(processes);

    let (tx, rx) = mpsc::channel();
    spawn_port_loader(tx.clone());

    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    let result = loop {
        while let Ok(msg) = rx.try_recv() {
            match msg {
                Msg::Ports(ports) => {
                    app.apply_ports(&ports);
                }
            }
        }

        terminal.draw(|frame| ui::draw(frame, &mut app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if handle_key(&mut app, key, &tx, &mut terminal)? {
                    break Ok(());
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }

        if app.should_quit {
            break Ok(());
        }
    };

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn spawn_port_loader(tx: mpsc::Sender<Msg>) {
    thread::spawn(move || {
        if let Ok(ports) = listening_ports_cached(false) {
            let _ = tx.send(Msg::Ports(ports));
        } else {
            let _ = tx.send(Msg::Ports(Vec::new()));
        }
    });
}

fn handle_key(
    app: &mut App,
    key: KeyEvent,
    tx: &mpsc::Sender<Msg>,
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> anyhow::Result<bool> {
    if key.kind != KeyEventKind::Press {
        return Ok(false);
    }
    if app.is_confirming_kill() {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Some(pending) = app.take_pending_kill() {
                    kill_selection(app, pending.force, pending.tree)?;
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.cancel_kill_confirm();
            }
            _ => {}
        }
        return Ok(false);
    }
    if is_kill_preview_key(key) {
        let (force, tree) = kill_preview_flags(key);
        request_kill_preview(app, terminal, force, tree)?;
        return Ok(false);
    }
    if key.code == KeyCode::Char('r') {
        crate::process::ports::clear_port_cache();
        app.refresh();
        app.status = "Refreshing processes + ports…".into();
        spawn_port_loader(tx.clone());
        return Ok(false);
    }
    Ok(handle_key_event(app, key))
}

fn is_kill_preview_key(key: KeyEvent) -> bool {
    if key.kind != KeyEventKind::Press {
        return false;
    }
    matches!(
        key.code,
        KeyCode::Char('k') | KeyCode::Char('K') | KeyCode::Char('t') | KeyCode::Char('T')
    )
}

fn kill_preview_flags(key: KeyEvent) -> (bool, bool) {
    match key.code {
        KeyCode::Char('K') => (true, false),
        KeyCode::Char('t') => (false, true),
        KeyCode::Char('T') => (true, true),
        _ => (false, false),
    }
}

/// Handle a key press without drawing. Returns `true` when the TUI should exit.
pub fn handle_key_event(app: &mut App, key: KeyEvent) -> bool {
    if key.kind != KeyEventKind::Press {
        return false;
    }
    let code = key.code;

    if app.searching {
        match code {
            KeyCode::Esc => {
                app.searching = false;
            }
            KeyCode::Enter => {
                app.searching = false;
            }
            KeyCode::Backspace => {
                app.query.pop();
                app.refilter();
            }
            KeyCode::Char(c) => {
                app.query.push(c);
                app.refilter();
            }
            _ => {}
        }
        return false;
    }

    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match code {
            KeyCode::Char('u') | KeyCode::Char('U') => {
                app.move_page_up(app.viewport_rows);
                return false;
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                app.move_page_down(app.viewport_rows);
                return false;
            }
            _ => {}
        }
    }

    match code {
        KeyCode::Char('q') => {
            app.should_quit = true;
            return true;
        }
        KeyCode::Esc => {
            if app.show_detail {
                app.show_detail = false;
            } else if app.expanded_project.is_some() {
                app.collapse_project();
            } else {
                app.should_quit = true;
                return true;
            }
        }
        KeyCode::Enter => {
            if app.in_project_list() {
                app.toggle_project_expand();
            } else {
                app.toggle_detail();
            }
        }
        KeyCode::Char('i') => app.toggle_detail(),
        KeyCode::Char('/') => {
            app.searching = true;
        }
        KeyCode::Up => app.move_up(),
        KeyCode::Down | KeyCode::Char('j') => app.move_down(),
        KeyCode::PageUp => app.move_page_up(app.viewport_rows),
        KeyCode::PageDown => app.move_page_down(app.viewport_rows),
        KeyCode::Char('g') => app.move_first(),
        KeyCode::Char('G') => app.move_last(),
        KeyCode::Char(' ') => app.toggle_select_current(),
        KeyCode::Char('k') | KeyCode::Char('K') | KeyCode::Char('t') | KeyCode::Char('T') => {
            let (force, tree) = kill_preview_flags(key);
            app.request_kill_confirm(force, tree);
        }
        KeyCode::Char('p') => app.toggle_ports_only(),
        KeyCode::Char('e') => app.toggle_tree_view(),
        KeyCode::Char('P') => app.toggle_project_view(),
        _ => {}
    }
    false
}

fn request_kill_preview(
    app: &mut App,
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    force: bool,
    tree: bool,
) -> anyhow::Result<()> {
    app.request_kill_confirm(force, tree);
    terminal.draw(|frame| ui::draw(frame, app))?;
    Ok(())
}

fn kill_selection(app: &mut App, force: bool, tree: bool) -> anyhow::Result<()> {
    let roots = app.pids_to_kill();
    if roots.is_empty() {
        app.status = "Nothing to kill".into();
        return Ok(());
    }

    let pids = if tree {
        collect_tree_pids(&app.processes, &roots)
    } else {
        roots
    };

    let mut killed = 0;
    let mut results = Vec::new();
    for pid in pids {
        let info = app.processes.iter().find(|p| p.pid == pid);
        let name = info.map(|p| p.name.clone()).unwrap_or_else(|| "?".into());
        let ports = info.map(|p| p.ports.clone()).unwrap_or_default();
        let mem = info.map(|p| p.memory_bytes).unwrap_or(0);

        let outcome = kill_pid(pid, &name, force)?;
        let signal = if force && matches!(outcome, KillOutcome::ForceKilled) {
            KillSignal::Kill
        } else {
            KillSignal::Term
        };
        let _ = append_entry(HistoryEntry::new(
            pid,
            &name,
            ports.clone(),
            signal,
            format!("{outcome:?}"),
        ));
        let result = crate::report::KillResult::new(mem, ports, outcome);
        if result.is_success() {
            killed += 1;
        }
        app.status = format!("{name} ({pid}): {:?}", result.outcome);
        results.push(result);
    }

    app.refresh();
    if let Ok(ports) = listening_ports_cached(false) {
        app.apply_ports(&ports);
    }
    let kind = if tree { "tree " } else { "" };
    let freed = crate::report::freed_bytes_from_results(&results);
    let released = crate::report::released_ports(&results);
    let mb = freed as f64 / (1024.0 * 1024.0);
    let ports_hint = if released.is_empty() {
        String::new()
    } else {
        let list = released
            .iter()
            .map(|p| format!(":{p}"))
            .collect::<Vec<_>>()
            .join(", ");
        format!("; ports {list}")
    };
    app.status =
        format!("Killed {killed} {kind}process(es); ~{mb:.0} MB freed (estimate){ports_hint}");
    let _ = io::Write::flush(&mut io::stdout());
    Ok(())
}
