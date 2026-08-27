use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use sweeper::process::ProcessInfo;
use sweeper::tui::App;

fn press(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn ctrl(code: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(code), KeyModifiers::CONTROL)
}

fn proc(pid: u32, name: &str, ports: Vec<u16>) -> ProcessInfo {
    ProcessInfo {
        pid,
        ppid: 1,
        name: name.into(),
        cpu: 0.0,
        memory_bytes: 1024,
        ports,
        command: None,
        cwd: None,
        run_time_secs: 0,
        is_zombie: false,
    }
}

fn handle_test_key(app: &mut App, key: KeyEvent) -> bool {
    if key.kind != KeyEventKind::Press {
        return false;
    }
    if app.is_confirming_kill() {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                app.take_pending_kill();
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.cancel_kill_confirm();
            }
            _ => {}
        }
        return false;
    }
    sweeper::tui::handle_key_event(app, key)
}

#[test]
fn g_and_g_jump_to_first_and_last_row() {
    let mut app = App::new(vec![
        proc(1, "a", vec![]),
        proc(2, "b", vec![]),
        proc(3, "c", vec![]),
    ]);
    handle_test_key(&mut app, press(KeyCode::Char('G')));
    assert_eq!(app.cursor, 2);
    handle_test_key(&mut app, press(KeyCode::Char('g')));
    assert_eq!(app.cursor, 0);
}

#[test]
fn page_up_and_down_move_by_viewport() {
    let mut app = App::new(vec![
        proc(1, "a", vec![]),
        proc(2, "b", vec![]),
        proc(3, "c", vec![]),
        proc(4, "d", vec![]),
    ]);
    app.set_viewport_rows(2);
    handle_test_key(&mut app, press(KeyCode::PageDown));
    assert_eq!(app.cursor, 2);
    handle_test_key(&mut app, press(KeyCode::PageUp));
    assert_eq!(app.cursor, 0);
    handle_test_key(&mut app, ctrl('d'));
    assert_eq!(app.cursor, 2);
    handle_test_key(&mut app, ctrl('u'));
    assert_eq!(app.cursor, 0);
}

#[test]
fn search_key_refilters_visible_rows() {
    let mut app = App::new(vec![proc(1, "node", vec![3000]), proc(2, "bash", vec![])]);
    handle_test_key(&mut app, press(KeyCode::Char('/')));
    assert!(app.searching);
    handle_test_key(&mut app, press(KeyCode::Char('n')));
    handle_test_key(&mut app, press(KeyCode::Char('o')));
    handle_test_key(&mut app, press(KeyCode::Char('d')));
    handle_test_key(&mut app, press(KeyCode::Char('e')));
    assert_eq!(app.filtered.len(), 1);
    assert_eq!(app.processes[app.filtered[0]].name, "node");
    handle_test_key(&mut app, press(KeyCode::Backspace));
    assert_eq!(app.query, "nod");
    assert_eq!(app.filtered.len(), 1);
}

#[test]
fn p_toggles_ports_only_filter() {
    let mut app = App::new(vec![proc(1, "node", vec![3000]), proc(2, "bash", vec![])]);
    handle_test_key(&mut app, press(KeyCode::Char('p')));
    assert!(app.ports_only);
    assert_eq!(app.filtered.len(), 1);
    handle_test_key(&mut app, press(KeyCode::Char('p')));
    assert!(!app.ports_only);
    assert_eq!(app.filtered.len(), 2);
}

#[test]
fn k_shows_kill_preview_in_status() {
    let mut app = App::new(vec![ProcessInfo {
        pid: 4812,
        ppid: 1,
        name: "node".into(),
        cpu: 0.0,
        memory_bytes: 0,
        ports: vec![3000],
        command: Some("./node_modules/.bin/vite".into()),
        cwd: None,
        run_time_secs: 0,
        is_zombie: false,
    }]);
    handle_test_key(&mut app, press(KeyCode::Char('k')));
    assert!(app.is_confirming_kill());
    assert!(app.status.contains("Confirm kill?"));
    assert!(app.status.contains("4812"));
    assert!(app.status.contains(":3000"));
    handle_test_key(&mut app, press(KeyCode::Char('n')));
    assert!(!app.is_confirming_kill());
    assert_eq!(app.status, "Kill cancelled");
}

#[test]
fn orbstack_key_opens_resources_when_available() {
    let mut app = App::new(vec![]);
    app.resource_snapshot.available = true;
    handle_test_key(&mut app, press(KeyCode::Char('o')));
    assert!(app.resources_open);
    handle_test_key(&mut app, press(KeyCode::Esc));
    assert!(!app.resources_open);
}

#[test]
fn question_mark_toggles_help_overlay() {
    let mut app = App::new(vec![proc(1, "node", vec![])]);
    assert!(!app.show_help_overlay);
    handle_test_key(&mut app, press(KeyCode::Char('?')));
    assert!(app.show_help_overlay);
    handle_test_key(&mut app, press(KeyCode::Esc));
    assert!(!app.show_help_overlay);
}

#[test]
fn clean_view_is_accessible_from_processes() {
    let mut app = App::new(vec![proc(1, "node", vec![])]);
    handle_test_key(&mut app, press(KeyCode::Char('c')));
    assert!(app.in_clean_list());
}

#[test]
fn draw_smoke_test_backend_layout() {
    let mut app = App::new(vec![
        proc(1, "node", vec![3000]),
        proc(2, "vite", vec![5173]),
    ]);
    app.refilter();
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal
        .draw(|frame| sweeper::tui::ui::draw(frame, &mut app))
        .expect("draw");
    let buffer = terminal.backend().buffer();
    assert_eq!(buffer.area().width, 100);
    assert_eq!(buffer.area().height, 30);
    let content = buffer
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();
    assert!(content.contains("Sweeper"));
    assert!(content.contains("node"));
}
