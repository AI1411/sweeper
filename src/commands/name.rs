use crate::history::{append_entry, HistoryEntry, KillSignal};
use crate::process::kill::{kill_pid, KillOutcome};
use crate::process::list::{find_by_name_fuzzy, list_processes};
use crate::process::plan::{plan_kills, print_dry_run};
use crate::process::tree::collect_tree_pids;
use crate::process::ProcessInfo;
use crate::report;
use crate::style;

use super::confirm::confirm;

pub fn run_name(query: &str, force: bool, tree: bool, dry_run: bool) -> anyhow::Result<()> {
    let matches = find_by_name_fuzzy(query);
    if matches.is_empty() {
        println!(
            "{}",
            style::warn(format!("No processes matching '{query}'"))
        );
        return Ok(());
    }

    let all = list_processes();
    let targets = expand_targets(&all, &matches, tree);

    if dry_run {
        let roots: Vec<u32> = targets.iter().map(|p| p.pid).collect();
        let planned = plan_kills(&all, &roots, tree);
        print_dry_run(&planned, tree);
        return Ok(());
    }

    println!(
        "{} {} processes{}\n",
        style::header("Found"),
        style::process_name(targets.len()),
        if tree {
            style::dim(" (including tree)")
        } else {
            String::new()
        }
    );
    for p in &targets {
        println!(
            "  {}  {}  {}  {}",
            style::pid(format!("{:>6}", p.pid)),
            style::process_name(&p.name),
            style::cpu(p.cpu),
            style::mem(format!("{:.0} MB", p.memory_mb()))
        );
    }
    let total: u64 = targets.iter().map(|p| p.memory_bytes).sum();
    println!(
        "\n{} {}",
        style::dim("Total memory:"),
        style::mem(format!("{:.1} GB", total as f64 / 1e9))
    );
    if !confirm("Kill all?")? {
        println!("{}", style::warn("Cancelled."));
        return Ok(());
    }
    let mut outcomes = Vec::new();
    for p in targets {
        let outcome = kill_one(&p, force)?;
        outcomes.push(report::KillResult::new(
            p.memory_bytes,
            p.ports.clone(),
            outcome,
        ));
    }
    report::print_kill_summary(&outcomes);
    Ok(())
}

fn expand_targets(all: &[ProcessInfo], matches: &[ProcessInfo], tree: bool) -> Vec<ProcessInfo> {
    if !tree {
        return matches.to_vec();
    }
    let roots: Vec<u32> = matches.iter().map(|p| p.pid).collect();
    let pids = collect_tree_pids(all, &roots);
    pids.iter()
        .filter_map(|pid| {
            all.iter()
                .find(|p| p.pid == *pid)
                .cloned()
                .or_else(|| matches.iter().find(|p| p.pid == *pid).cloned())
        })
        .collect()
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
