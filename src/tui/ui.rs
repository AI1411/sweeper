use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table},
    Frame,
};

use super::app::App;

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(frame.area());

    draw_search(frame, app, chunks[0]);
    draw_table(frame, app, chunks[1]);
    draw_footer(frame, app, chunks[2]);
}

fn draw_search(frame: &mut Frame, app: &App, area: Rect) {
    let title = if app.searching {
        " Search (Esc to leave) "
    } else {
        " Sweeper "
    };
    let text = if app.searching || !app.query.is_empty() {
        format!("/{}", app.query)
    } else {
        "Press / to search".to_string()
    };
    let widget = Paragraph::new(text).block(Block::default().borders(Borders::ALL).title(title));
    frame.render_widget(widget, area);
}

fn draw_table(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["", "PID", "PROCESS", "PORT", "CPU", "MEM"])
        .style(Style::default().add_modifier(Modifier::BOLD));

    let rows = app.filtered.iter().enumerate().map(|(idx, &pi)| {
        let p = &app.processes[pi];
        let mark = if app.selected.contains(&p.pid) {
            "*"
        } else if idx == app.cursor {
            ">"
        } else {
            " "
        };
        let ports = if p.ports.is_empty() {
            "-".to_string()
        } else {
            p.ports
                .iter()
                .map(|port| format!(":{port}"))
                .collect::<Vec<_>>()
                .join(",")
        };
        let row = Row::new(vec![
            mark.to_string(),
            p.pid.to_string(),
            p.name.clone(),
            ports,
            format!("{:.1}%", p.cpu),
            format!("{:.0} MB", p.memory_mb()),
        ]);
        if idx == app.cursor {
            row.style(Style::default().add_modifier(Modifier::REVERSED))
        } else {
            row
        }
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(2),
            Constraint::Length(8),
            Constraint::Percentage(35),
            Constraint::Length(14),
            Constraint::Length(8),
            Constraint::Length(10),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Processes "));

    frame.render_widget(table, area);
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let help = Line::from(vec![
        Span::raw("[↑↓] Move  "),
        Span::raw("[Space] Select  "),
        Span::raw("[k] Kill  "),
        Span::raw("[K] Force  "),
        Span::raw("[/] Search  "),
        Span::raw("[q] Quit"),
    ]);
    let status = if app.status.is_empty() {
        format!("{} processes", app.filtered.len())
    } else {
        app.status.clone()
    };
    let text = vec![help, Line::from(status)];
    let widget = Paragraph::new(text).block(Block::default().borders(Borders::ALL).title(" Help "));
    frame.render_widget(widget, area);
}
