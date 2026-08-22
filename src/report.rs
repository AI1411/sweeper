use crate::process::kill::KillOutcome;
use crate::style;

/// Sum memory for successful kills only (estimate from pre-kill snapshot).
pub fn freed_bytes(outcomes: &[(u64, KillOutcome)]) -> u64 {
    outcomes
        .iter()
        .filter(|(_, o)| matches!(o, KillOutcome::Terminated | KillOutcome::ForceKilled))
        .map(|(bytes, _)| *bytes)
        .sum()
}

pub fn print_summary(count: usize, bytes: u64) {
    if count == 0 {
        return;
    }
    println!(
        "\n{} {}",
        style::success(format!("Terminated {count} process(es)")),
        style::dim("(from last snapshot)")
    );
    println!(
        "{} {}",
        style::dim("Estimated memory freed:"),
        style::mem(format!("{:.0} MB", bytes as f64 / (1024.0 * 1024.0)))
    );
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
}
