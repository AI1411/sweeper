use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use super::app::App;

const ACCENT: Color = Color::Cyan;
const PORT: Color = Color::Yellow;
const MEM: Color = Color::Magenta;
const MUTED: Color = Color::DarkGray;

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
        Line::from(vec![
            Span::styled(
                "/",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::raw(app.query.clone()),
        ])
    } else {
        Line::from(Span::styled(
            "Press / to search",
            Style::default().fg(MUTED),
        ))
    };
    let widget = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ACCENT))
            .title(Span::styled(
                title,
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            )),
    );
    frame.render_widget(widget, area);
}

fn draw_table(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["", "PID", "PROCESS", "PORT", "CPU", "MEM"])
        .style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD));

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
        let cpu_style = if p.cpu >= 50.0 {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        } else if p.cpu >= 20.0 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Green)
        };
        let row = Row::new(vec![
            Cell::from(mark),
            Cell::from(p.pid.to_string()).style(Style::default().fg(MUTED)),
            Cell::from(p.name.clone()).style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from(ports).style(Style::default().fg(PORT)),
            Cell::from(format!("{:.1}%", p.cpu)).style(cpu_style),
            Cell::from(format!("{:.0} MB", p.memory_mb())).style(Style::default().fg(MEM)),
        ]);
        if idx == app.cursor {
            row.style(
                Style::default()
                    .bg(ACCENT)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
        } else if app.selected.contains(&p.pid) {
            row.style(Style::default().fg(Color::LightCyan))
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
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ACCENT))
            .title(Span::styled(
                " Processes ",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            )),
    );

    frame.render_widget(table, area);
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let help = Line::from(vec![
        Span::styled(
            "[↑↓]",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Move  "),
        Span::styled(
            "[Space]",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Select  "),
        Span::styled(
            "[k]",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Kill  "),
        Span::styled(
            "[K]",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Force  "),
        Span::styled(
            "[t]",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Tree  "),
        Span::styled(
            "[T]",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Force tree  "),
        Span::styled(
            "[/]",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Search  "),
        Span::styled(
            "[q]",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Quit"),
    ]);
    let status = if app.status.is_empty() {
        format!("{} processes", app.filtered.len())
    } else {
        app.status.clone()
    };
    let text = vec![
        help,
        Line::from(Span::styled(status, Style::default().fg(MUTED))),
    ];
    let widget = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(MUTED))
            .title(Span::styled(" Help ", Style::default().fg(MUTED))),
    );
    frame.render_widget(widget, area);
}
