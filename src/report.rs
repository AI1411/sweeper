use crate::process::kill::KillOutcome;
use crate::style;

/// One kill attempt with pre-kill snapshot fields for summary reporting.
#[derive(Debug, Clone)]
pub struct KillResult {
    pub memory_bytes: u64,
    pub ports: Vec<u16>,
    pub outcome: KillOutcome,
}

impl KillResult {
    pub fn new(memory_bytes: u64, ports: Vec<u16>, outcome: KillOutcome) -> Self {
        Self {
            memory_bytes,
            ports,
            outcome,
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(
            self.outcome,
            KillOutcome::Terminated | KillOutcome::ForceKilled
        )
    }
}

/// Sum memory for successful kills only (estimate from pre-kill snapshot).
pub fn freed_bytes(outcomes: &[(u64, KillOutcome)]) -> u64 {
    outcomes
        .iter()
        .filter(|(_, o)| matches!(o, KillOutcome::Terminated | KillOutcome::ForceKilled))
        .map(|(bytes, _)| *bytes)
        .sum()
}

pub fn freed_bytes_from_results(results: &[KillResult]) -> u64 {
    results
        .iter()
        .filter(|r| r.is_success())
        .map(|r| r.memory_bytes)
        .sum()
}

/// Unique ports from successfully killed processes, sorted numerically.
pub fn released_ports(results: &[KillResult]) -> Vec<u16> {
    let mut ports: Vec<u16> = results
        .iter()
        .filter(|r| r.is_success())
        .flat_map(|r| r.ports.iter().copied())
        .collect();
    ports.sort_unstable();
    ports.dedup();
    ports
}

pub fn print_summary(count: usize, bytes: u64) {
    print_kill_summary_from_parts(count, bytes, &[]);
}

pub fn print_kill_summary(results: &[KillResult]) {
    let formatted = format_kill_summary(results);
    if !formatted.is_empty() {
        print!("{formatted}");
    }
}

pub fn format_kill_summary(results: &[KillResult]) -> String {
    let count = results.iter().filter(|r| r.is_success()).count();
    if count == 0 {
        return String::new();
    }
    let bytes = freed_bytes_from_results(results);
    let ports = released_ports(results);
    format_kill_summary_from_parts(count, bytes, &ports)
}

fn print_kill_summary_from_parts(count: usize, bytes: u64, ports: &[u16]) {
    let formatted = format_kill_summary_from_parts(count, bytes, ports);
    if !formatted.is_empty() {
        print!("{formatted}");
    }
}

fn format_kill_summary_from_parts(count: usize, bytes: u64, ports: &[u16]) -> String {
    if count == 0 {
        return String::new();
    }
    let mut out = String::new();
    use std::fmt::Write;
    writeln!(
        out,
        "\n{} {}",
        style::success(format!("Terminated {count} process(es)")),
        style::dim("(from last snapshot)")
    )
    .unwrap();
    writeln!(
        out,
        "{} {}",
        style::dim("Estimated memory freed:"),
        style::mem(format!("{:.0} MB", bytes as f64 / (1024.0 * 1024.0)))
    )
    .unwrap();
    if ports.is_empty() {
        return out;
    }
    writeln!(out).unwrap();
    writeln!(out, "{}", style::header("Ports released:")).unwrap();
    for port in ports {
        writeln!(out, "  {}", style::port(format!(":{port}"))).unwrap();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sums_only_successful() {
        let rows = [
            (100u64, KillOutcome::Terminated),
            (50, KillOutcome::StillAlive),
            (25, KillOutcome::ForceKilled),
            (10, KillOutcome::SkippedProtected),
        ];
        assert_eq!(freed_bytes(&rows), 125);
    }

    #[test]
    fn released_ports_dedupes_and_sorts() {
        let results = [
            KillResult::new(100, vec![3000, 5173], KillOutcome::Terminated),
            KillResult::new(50, vec![3000], KillOutcome::ForceKilled),
            KillResult::new(10, vec![8080], KillOutcome::StillAlive),
        ];
        assert_eq!(released_ports(&results), vec![3000, 5173]);
        assert_eq!(freed_bytes_from_results(&results), 150);
    }

    #[test]
    fn released_ports_empty_when_no_success() {
        let results = [KillResult::new(100, vec![3000], KillOutcome::StillAlive)];
        assert!(released_ports(&results).is_empty());
    }
}
