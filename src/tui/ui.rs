use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use super::app::{App, ViewMode};
use crate::project::summarize_group;

const ACCENT: Color = Color::Cyan;
const PORT: Color = Color::Yellow;
const MEM: Color = Color::Magenta;
const MUTED: Color = Color::DarkGray;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let detail_rows = if app.show_detail { 5 } else { 0 };
    let search_rows = if app.resources_open || !app.resource_snapshot.available {
        3
    } else if app.resource_snapshot.available {
        6
    } else {
        3
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(search_rows),
            Constraint::Min(5),
            Constraint::Length(detail_rows),
            Constraint::Length(3),
        ])
        .split(frame.area());

    draw_search(frame, app, chunks[0]);
    draw_table(frame, app, chunks[1]);
    if app.show_detail && chunks.len() > 3 {
        draw_detail(frame, app, chunks[2]);
        draw_footer(frame, app, chunks[3]);
    } else {
        draw_footer(frame, app, chunks[2]);
    }
}

fn draw_search(frame: &mut Frame, app: &App, area: Rect) {
    let title = if app.searching {
        " Search (Esc to leave) "
    } else if app.resources_open {
        " OrbStack / Docker "
    } else {
        " Sweeper "
    };
    let mut line_spans = Vec::new();
    if app.searching || !app.query.is_empty() {
        line_spans.push(Line::from(vec![
            Span::styled(
                "/",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::raw(app.query.clone()),
        ]));
    } else if !app.resources_open {
        line_spans.push(Line::from(Span::styled(
            "Press / to search",
            Style::default().fg(MUTED),
        )));
    }
    if !app.resources_open && app.resource_snapshot.available {
        if let Some(line) = app.resource_snapshot.summary_line() {
            line_spans.push(Line::from(Span::styled(line, Style::default().fg(MEM))));
        }
        if let Some(line) = app.resource_snapshot.disk_summary_line() {
            line_spans.push(Line::from(Span::styled(line, Style::default().fg(MUTED))));
        }
        line_spans.push(Line::from(Span::styled(
            "Press o for OrbStack details",
            Style::default().fg(MUTED),
        )));
    }
    let widget = Paragraph::new(line_spans).block(
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

fn draw_table(frame: &mut Frame, app: &mut App, area: Rect) {
    let viewport = area.height.saturating_sub(3) as usize;
    app.set_viewport_rows(viewport);

    if app.resources_open {
        draw_resources_panel(frame, app, area);
        return;
    }

    if app.in_project_list() {
        draw_project_table(frame, app, area);
        return;
    }

    let header = Row::new(vec!["", "PID", "PROCESS", "PORT", "CPU", "MEM"])
        .style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = if app.tree_view {
        app.tree_rows
            .iter()
            .map(|tr| process_row(&app.processes[tr.process_index], app, &tr.prefix))
            .collect()
    } else {
        app.filtered
            .iter()
            .map(|&pi| process_row(&app.processes[pi], app, ""))
            .collect()
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(2),
            Constraint::Length(8),
            Constraint::Percentage(30),
            Constraint::Length(18),
            Constraint::Length(8),
            Constraint::Length(10),
        ],
    )
    .header(header)
    .highlight_symbol(">")
    .row_highlight_style(
        Style::default()
            .bg(ACCENT)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ACCENT))
            .title(Span::styled(
                table_title(app),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            )),
    );

    frame.render_stateful_widget(table, area, &mut app.table_state);
}

fn draw_resources_panel(frame: &mut Frame, app: &App, area: Rect) {
    let lines: Vec<Line> = app
        .resource_lines()
        .into_iter()
        .map(|l| Line::from(Span::raw(l)))
        .collect();
    let title = match app.resource_panel {
        super::resources::ResourcePanel::Summary => " OrbStack summary ",
        super::resources::ResourcePanel::Containers => " Containers ",
        super::resources::ResourcePanel::Docker => " Docker disk ",
    };
    let widget = Paragraph::new(lines).block(
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

fn draw_project_table(frame: &mut Frame, app: &mut App, area: Rect) {
    let header = Row::new(vec!["PROJECT", "PATH", "PROCS", "MEM", "PORTS"])
        .style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = app
        .visible_project_groups()
        .iter()
        .map(|g| {
            let s = summarize_group(g);
            let ports = if s.ports.is_empty() {
                "-".to_string()
            } else {
                s.ports
                    .iter()
                    .map(|port| format!(":{port}"))
                    .collect::<Vec<_>>()
                    .join(",")
            };
            let mem_mb = s.memory_bytes as f64 / (1024.0 * 1024.0);
            Row::new(vec![
                Cell::from(g.name.clone()).style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from(g.path.clone()).style(Style::default().fg(MUTED)),
                Cell::from(s.process_count.to_string()),
                Cell::from(format!("{:.0} MB", mem_mb)).style(Style::default().fg(MEM)),
                Cell::from(ports).style(Style::default().fg(PORT)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(16),
            Constraint::Percentage(40),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(18),
        ],
    )
    .header(header)
    .highlight_symbol(">")
    .row_highlight_style(
        Style::default()
            .bg(ACCENT)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ACCENT))
            .title(Span::styled(
                project_table_title(app),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            )),
    );

    frame.render_stateful_widget(table, area, &mut app.table_state);
}

fn process_row<'a>(p: &crate::process::ProcessInfo, app: &App, prefix: &str) -> Row<'a> {
    let mark = if app.selected.contains(&p.pid) {
        "*"
    } else {
        " "
    };
    let ports = if p.ports.is_empty() {
        "-".into()
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
    let name = if prefix.is_empty() {
        p.name.clone()
    } else {
        format!("{prefix}{}", p.name)
    };
    let row = Row::new(vec![
        Cell::from(mark),
        Cell::from(p.pid.to_string()).style(Style::default().fg(MUTED)),
        Cell::from(name).style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from(ports).style(Style::default().fg(PORT)),
        Cell::from(format!("{:.1}%", p.cpu)).style(cpu_style),
        Cell::from(format!("{:.0} MB", p.memory_mb())).style(Style::default().fg(MEM)),
    ]);
    if app.selected.contains(&p.pid) {
        row.style(Style::default().fg(Color::LightCyan))
    } else {
        row
    }
}

fn table_title(app: &App) -> String {
    let total = if app.tree_view {
        app.tree_rows.len()
    } else {
        app.filtered.len()
    };
    if app.view_mode == ViewMode::Projects && app.expanded_project.is_some() {
        let name = app
            .expanded_project
            .and_then(|idx| app.project_groups.get(idx))
            .map(|g| g.name.as_str())
            .unwrap_or("project");
        if total == 0 {
            format!(" {name} members ")
        } else {
            format!(" {name} members [{} / {}] ", app.cursor + 1, total)
        }
    } else {
        let mode = if app.tree_view { " tree " } else { " " };
        if total == 0 {
            format!(" Processes{mode}")
        } else {
            format!(" Processes{mode}[{} / {}] ", app.cursor + 1, total)
        }
    }
}

fn project_table_title(app: &App) -> String {
    let total = app.visible_project_groups().len();
    if total == 0 {
        " Projects ".to_string()
    } else {
        format!(" Projects [{} / {}] ", app.cursor + 1, total)
    }
}

fn draw_detail(frame: &mut Frame, app: &App, area: Rect) {
    let lines: Vec<Line> = app
        .format_process_detail()
        .into_iter()
        .map(|l| Line::from(Span::styled(l, Style::default().fg(MUTED))))
        .collect();
    let widget = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ACCENT))
            .title(Span::styled(
                " Detail (i/Esc) ",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            )),
    );
    frame.render_widget(widget, area);
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let help = Line::from(vec![
        Span::styled(
            "[↑↓]",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Move  "),
        Span::styled(
            "[g/G]",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Jump  "),
        Span::styled(
            "[PgUp/PgDn]",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Page  "),
        Span::styled(
            "[Space]",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Select  "),
        Span::styled(
            "[k]",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Kill→y  "),
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
            "[p]",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Ports  "),
        Span::styled(
            "[e]",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Tree view  "),
        Span::styled(
            "[P]",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Projects  "),
        Span::styled(
            "[o]",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" OrbStack  "),
        Span::styled(
            "[i]",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Detail  "),
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
        if app.in_project_list() {
            format!("{} projects", app.visible_project_groups().len())
        } else {
            format!("{} processes", app.filtered.len())
        }
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
