use std::io::{self, Write};

use crate::commands::confirm::confirm;
use crate::process::ProcessInfo;
use crate::style;

/// How `high` / `h` is resolved in [`parse_selection`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseHigh<'a> {
    /// Keyword `high` is invalid.
    None,
    /// 0-based indices of high-confidence rows.
    Indices(&'a [usize]),
}

/// Parse a 1-based selection string into 0-based indices.
///
/// `Some(vec![])` is cancel. `None` is invalid input.
pub fn parse_selection(input: &str, total: usize, high: ParseHigh<'_>) -> Option<Vec<usize>> {
    let trimmed = input.trim();
    let key = trimmed.to_ascii_lowercase();
    match key.as_str() {
        "" | "q" | "quit" | "n" | "no" => return Some(Vec::new()),
        "all" | "a" | "*" => return Some((0..total).collect()),
        "high" | "h" => {
            return match high {
                ParseHigh::Indices(idx) => {
                    let mut v: Vec<usize> = idx.iter().copied().filter(|&i| i < total).collect();
                    v.sort_unstable();
                    v.dedup();
                    Some(v)
                }
                ParseHigh::None => None,
            };
        }
        _ => {}
    }

    let normalized = trimmed.replace(',', " ");
    let mut indices = Vec::new();
    for part in normalized.split_whitespace() {
        for idx in parse_part(part, total)? {
            if !indices.contains(&idx) {
                indices.push(idx);
            }
        }
    }
    if indices.is_empty() {
        return None;
    }
    indices.sort_unstable();
    Some(indices)
}

fn parse_part(part: &str, total: usize) -> Option<Vec<usize>> {
    if let Some((start, end)) = part.split_once('-') {
        let start: usize = start.parse().ok()?;
        let end: usize = end.parse().ok()?;
        if start == 0 || end == 0 || start > end || end > total {
            return None;
        }
        return Some((start - 1..=end - 1).collect());
    }
    let n: usize = part.parse().ok()?;
    if n == 0 || n > total {
        return None;
    }
    Some(vec![n - 1])
}

/// Confirm all, or pick a subset with the shared selection grammar.
pub fn confirm_or_pick(
    total: usize,
    all_question: &str,
    high: ParseHigh<'_>,
) -> io::Result<Vec<usize>> {
    if total == 0 {
        return Ok(Vec::new());
    }
    if confirm(all_question)? {
        return Ok((0..total).collect());
    }
    if total == 1 {
        return Ok(Vec::new());
    }
    let hint = match high {
        ParseHigh::Indices(idx) if !idx.is_empty() => {
            format!("Kill which? (1-{total}, 1-3, all, high, q)")
        }
        _ => format!("Kill which? (1-{total}, 1-3, all, q)"),
    };
    print!("{} {} ", hint, style::dim("[1,2,...]"));
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    match parse_selection(&buf, total, high) {
        Some(indices) => Ok(indices),
        None => {
            println!("{}", style::warn("Invalid selection."));
            Ok(Vec::new())
        }
    }
}

pub fn format_process_header() -> String {
    format!(
        "     {}  {}  {}  {}  {}  {}",
        style::header(pad("PID", 6)),
        style::header(pad("PROCESS", 12)),
        style::header(pad("PORT", 12)),
        style::header(pad("PROJECT", 14)),
        style::header("CPU"),
        style::header("MEM")
    )
}

pub fn format_process_row(n: usize, p: &ProcessInfo) -> String {
    let ports = format_ports(&p.ports);
    let project = crate::project::infer_project(p)
        .map(|(name, _)| name)
        .unwrap_or_else(|| "-".into());
    format!(
        "{:>3}. {}  {}  {}  {}  {}  {}",
        n,
        style::pid(pad(&p.pid.to_string(), 6)),
        style::process_name(pad_trunc(&p.name, 12)),
        style::port(pad_trunc(&ports, 12)),
        style::dim(pad_trunc(&project, 14)),
        style::cpu(p.cpu),
        style::mem(format!("{:.0} MB", p.memory_mb()))
    )
}

pub fn print_process_table(items: &[ProcessInfo]) {
    println!("{}", format_process_header());
    for (i, p) in items.iter().enumerate() {
        println!("{}", format_process_row(i + 1, p));
    }
}

fn format_ports(ports: &[u16]) -> String {
    if ports.is_empty() {
        return "-".into();
    }
    ports
        .iter()
        .map(|port| format!(":{port}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn pad(s: &str, width: usize) -> String {
    format!("{s:<width$}")
}

fn pad_trunc(s: &str, width: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    let display: String = if chars.len() > width {
        let mut t: String = chars.iter().take(width.saturating_sub(1)).collect();
        t.push('…');
        t
    } else {
        s.to_string()
    };
    format!("{display:<width$}")
}
