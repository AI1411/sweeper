mod app;
mod ui;

use std::io::{self, stdout};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;

use crate::history::{append_entry, HistoryEntry, KillSignal};
use crate::process::kill::{kill_pid, KillOutcome};
use crate::process::list::list_processes;
use crate::process::ports::listening_ports;

use self::app::App;

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
    thread::spawn(move || {
        if let Ok(ports) = listening_ports() {
            let _ = tx.send(Msg::Ports(ports));
        }
    });

    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    let result = loop {
        while let Ok(msg) = rx.try_recv() {
            match msg {
                Msg::Ports(ports) => {
                    app.apply_ports(&ports);
                    app.status = format!("Loaded {} listening ports", ports.len());
                }
            }
        }

        terminal.draw(|frame| ui::draw(frame, &app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if handle_key(&mut app, key.code)? {
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

fn handle_key(app: &mut App, code: KeyCode) -> anyhow::Result<bool> {
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
        return Ok(false);
    }

    match code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
            return Ok(true);
        }
        KeyCode::Char('/') => {
            app.searching = true;
        }
        KeyCode::Up => app.move_up(),
        KeyCode::Down | KeyCode::Char('j') => app.move_down(),
        KeyCode::Char(' ') => app.toggle_select_current(),
        KeyCode::Char('k') => kill_selection(app, false)?,
        KeyCode::Char('K') => kill_selection(app, true)?,
        KeyCode::Char('r') => {
            app.refresh();
            app.status = "Refreshed process list".into();
        }
        _ => {}
    }
    Ok(false)
}

fn kill_selection(app: &mut App, force: bool) -> anyhow::Result<()> {
    let pids = app.pids_to_kill();
    if pids.is_empty() {
        app.status = "Nothing to kill".into();
        return Ok(());
    }

    let mut killed = 0;
    for pid in pids {
        let name = app
            .processes
            .iter()
            .find(|p| p.pid == pid)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "?".into());
        let ports = app
            .processes
            .iter()
            .find(|p| p.pid == pid)
            .map(|p| p.ports.clone())
            .unwrap_or_default();

        // Leave TUI briefly is hard; kill inline and report status.
        let outcome = kill_pid(pid, &name, force)?;
        let signal = if force && matches!(outcome, KillOutcome::ForceKilled) {
            KillSignal::Kill
        } else {
            KillSignal::Term
        };
        let _ = append_entry(HistoryEntry::new(
            pid,
            &name,
            ports,
            signal,
            format!("{outcome:?}"),
        ));
        if matches!(
            outcome,
            KillOutcome::Terminated | KillOutcome::ForceKilled
        ) {
            killed += 1;
        }
        app.status = format!("{name} ({pid}): {outcome:?}");
    }

    app.refresh();
    if let Ok(ports) = listening_ports() {
        app.apply_ports(&ports);
    }
    app.status = format!("Killed {killed} process(es)");
    let _ = io::Write::flush(&mut io::stdout());
    Ok(())
}
