use std::io::{self, Write};

use crate::history::{append_entry, HistoryEntry, KillSignal};
use crate::process::kill::{kill_pid, KillOutcome};
use crate::process::list::list_processes;
use crate::process::ProcessInfo;
use crate::report;
use crate::style;

use super::confirm::confirm;

pub fn run_top(force: bool, tree: bool, dry_run: bool) -> anyhow::Result<()> {
    let procs = list_processes();
    let cpu_leaders = top_by_cpu(&procs, 10);
    let mem_leaders = top_by_memory(&procs, 10);

    println!("{}\n", style::header("CPU"));
    for (i, p) in cpu_leaders.iter().enumerate() {
        print_leader(i + 1, p);
    }
    println!("\n{}\n", style::header("MEMORY"));
    for (i, p) in mem_leaders.iter().enumerate() {
        print_leader(i + 1, p);
    }

    let _ = tree;
    if dry_run {
        println!(
            "{}",
            style::dim("Dry run: omit --dry-run to kill by rank interactively.")
        );
        return Ok(());
    }

    print!("{} ", style::dim("Kill by rank [1-10], or q to skip:"));
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let choice = buf.trim();
    if choice.eq_ignore_ascii_case("q") || choice.is_empty() {
        println!("{}", style::warn("Skipped."));
        return Ok(());
    }
    let rank: usize = choice.parse().unwrap_or(0);
    if !(1..=10).contains(&rank) {
        println!("{}", style::warn("Invalid rank."));
        return Ok(());
    }

    let pick = cpu_leaders
        .get(rank - 1)
        .or_else(|| mem_leaders.get(rank - 1));
    let p = match pick {
        Some(p) => p,
        None => {
            println!("{}", style::warn("No process at that rank."));
            return Ok(());
        }
    };

    if !confirm(&format!("Kill {} (pid {})?", p.name, p.pid))? {
        println!("{}", style::warn("Cancelled."));
        return Ok(());
    }

    let outcome = kill_one(p, force)?;
    report::print_kill_summary(&[report::KillResult::new(
        p.memory_bytes,
        p.ports.clone(),
        outcome,
    )]);
    Ok(())
}

fn top_by_cpu(procs: &[ProcessInfo], n: usize) -> Vec<ProcessInfo> {
    let mut sorted = procs.to_vec();
    sorted.sort_by(|a, b| {
        b.cpu
            .partial_cmp(&a.cpu)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    sorted.into_iter().take(n).collect()
}

fn top_by_memory(procs: &[ProcessInfo], n: usize) -> Vec<ProcessInfo> {
    let mut sorted = procs.to_vec();
    sorted.sort_by_key(|p| std::cmp::Reverse(p.memory_bytes));
    sorted.into_iter().take(n).collect()
}

fn print_leader(rank: usize, p: &ProcessInfo) {
    println!(
        "{} {}  pid {}  {}",
        style::rank(rank),
        style::process_name(&p.name),
        style::pid(p.pid),
        style::mem(format!("{:.0} MB", p.memory_mb()))
    );
}

fn kill_one(p: &ProcessInfo, force: bool) -> anyhow::Result<KillOutcome> {
    let mut use_force = force;
    let mut outcome = kill_pid(p.pid, &p.name, use_force)?;
    if outcome == KillOutcome::StillAlive
        && !use_force
        && confirm(&format!("PID {} still alive. Force kill?", p.pid))?
    {
        use_force = true;
        outcome = kill_pid(p.pid, &p.name, true)?;
    }
    let signal = if use_force && outcome == KillOutcome::ForceKilled {
        KillSignal::Kill
    } else {
        KillSignal::Term
    };
    let _ = append_entry(HistoryEntry::new(
        p.pid,
        &p.name,
        p.ports.clone(),
        signal,
        format!("{outcome:?}"),
    ));
    println!(
        "{} {} {}: {}",
        style::process_name(&p.name),
        style::dim("pid"),
        style::pid(p.pid),
        style::kill_outcome(outcome)
    );
    Ok(outcome)
}
