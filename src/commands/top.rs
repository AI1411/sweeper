use std::io::{self, Write};

use crate::history::{append_entry, entry_for_process, KillSignal};
use crate::process::kill::{kill_pid, KillOutcome};
use crate::process::list::list_processes;
use crate::process::tree::collect_tree_pids;
use crate::process::ProcessInfo;
use crate::report;
use crate::style;

use super::confirm::confirm;

pub fn run_top(force: bool, tree: bool, dry_run: bool, json: bool) -> anyhow::Result<()> {
    let procs = list_processes();
    let cpu_leaders = top_by_cpu(&procs, 10);
    let mem_leaders = top_by_memory(&procs, 10);

    if json {
        return crate::json_output::emit_json(&crate::json_output::TopJson::from_leaders(
            &cpu_leaders,
            &mem_leaders,
        ));
    }

    println!("{}\n", style::header("CPU"));
    for (i, p) in cpu_leaders.iter().enumerate() {
        print_leader(i + 1, p, false);
    }
    println!("\n{}\n", style::header("MEMORY"));
    for (i, p) in mem_leaders.iter().enumerate() {
        print_leader(i + 1, p, true);
    }

    if dry_run {
        println!(
            "{}",
            style::dim("Dry run: omit --dry-run to kill by rank or PID interactively.")
        );
        return Ok(());
    }

    print!(
        "{} ",
        style::dim("Kill by CPU rank [1-10], memory rank [m1-m10], PID, or q:")
    );
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let choice = buf.trim();
    if choice.eq_ignore_ascii_case("q") || choice.is_empty() {
        println!("{}", style::warn("Skipped."));
        return Ok(());
    }

    let pick = resolve_selection(choice, &cpu_leaders, &mem_leaders, &procs);
    let p = match pick {
        Some(p) => p,
        None => {
            println!("{}", style::warn("Invalid selection."));
            return Ok(());
        }
    };

    let targets = expand_targets(&procs, p, tree);
    if targets.is_empty() {
        println!("{}", style::warn("No kill targets."));
        return Ok(());
    }

    if !confirm(&format!(
        "Kill {}{}?",
        p.name,
        if tree && targets.len() > 1 {
            format!(" tree ({} processes)", targets.len())
        } else {
            format!(" (pid {})", p.pid)
        }
    ))? {
        println!("{}", style::warn("Cancelled."));
        return Ok(());
    }

    let mut outcomes = Vec::new();
    for target in targets {
        let outcome = kill_one(&target, force)?;
        outcomes.push(report::KillResult::new(
            target.memory_bytes,
            target.ports.clone(),
            outcome,
        ));
    }
    report::print_kill_summary(&outcomes);
    Ok(())
}

/// Sort processes by CPU descending and take the top `n`.
pub fn top_by_cpu(procs: &[ProcessInfo], n: usize) -> Vec<ProcessInfo> {
    let mut sorted = procs.to_vec();
    sorted.sort_by(|a, b| {
        b.cpu
            .partial_cmp(&a.cpu)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    sorted.into_iter().take(n).collect()
}

/// Sort processes by memory descending and take the top `n`.
pub fn top_by_memory(procs: &[ProcessInfo], n: usize) -> Vec<ProcessInfo> {
    let mut sorted = procs.to_vec();
    sorted.sort_by_key(|p| std::cmp::Reverse(p.memory_bytes));
    sorted.into_iter().take(n).collect()
}

/// Resolve interactive selection: CPU rank `1-10`, memory rank `m1-m10`, or PID.
pub fn resolve_selection<'a>(
    choice: &str,
    cpu_leaders: &'a [ProcessInfo],
    mem_leaders: &'a [ProcessInfo],
    all: &'a [ProcessInfo],
) -> Option<&'a ProcessInfo> {
    let lower = choice.to_ascii_lowercase();
    if let Some(rest) = lower.strip_prefix('m') {
        let rank: usize = rest.parse().ok()?;
        return mem_leaders.get(rank.checked_sub(1)?);
    }
    if let Ok(rank) = lower.parse::<usize>() {
        if (1..=10).contains(&rank) {
            return cpu_leaders.get(rank - 1);
        }
        return all.iter().find(|p| p.pid == rank as u32);
    }
    None
}

fn expand_targets(all: &[ProcessInfo], root: &ProcessInfo, tree: bool) -> Vec<ProcessInfo> {
    if !tree {
        return vec![root.clone()];
    }
    let pids = collect_tree_pids(all, &[root.pid]);
    pids.iter()
        .filter_map(|pid| all.iter().find(|p| p.pid == *pid).cloned())
        .collect()
}

fn print_leader(rank: usize, p: &ProcessInfo, memory_list: bool) {
    let label = if memory_list {
        format!("m{rank}.")
    } else {
        format!("{rank}.")
    };
    println!(
        "{} {}  pid {}  {}",
        style::dim(label),
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
    let _ = append_entry(entry_for_process(
        p.pid,
        &p.name,
        p.ports.clone(),
        signal,
        format!("{outcome:?}"),
        Some(p),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn proc(pid: u32, cpu: f32, mem: u64) -> ProcessInfo {
        ProcessInfo {
            pid,
            ppid: 1,
            name: format!("p{pid}"),
            cpu,
            memory_bytes: mem,
            ports: vec![],
            command: None,
            cwd: None,
            run_time_secs: 0,
            is_zombie: false,
        }
    }

    #[test]
    fn top_json_shape() {
        let procs = vec![proc(1, 5.0, 100), proc(2, 1.0, 500)];
        let cpu = top_by_cpu(&procs, 10);
        let mem = top_by_memory(&procs, 10);
        let json = crate::json_output::TopJson::from_leaders(&cpu, &mem);
        let text = serde_json::to_string(&json).expect("serialize");
        let parsed: serde_json::Value = serde_json::from_str(&text).expect("parse");
        assert_eq!(parsed["cpu"][0]["rank"], 1);
        assert_eq!(parsed["cpu"][0]["pid"], 1);
        assert_eq!(parsed["memory"][0]["pid"], 2);
    }

    #[test]
    fn top_by_cpu_orders_descending() {
        let procs = vec![proc(1, 1.0, 100), proc(2, 5.0, 100), proc(3, 3.0, 100)];
        let top = top_by_cpu(&procs, 2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].pid, 2);
        assert_eq!(top[1].pid, 3);
    }

    #[test]
    fn top_by_memory_orders_descending() {
        let procs = vec![proc(1, 0.0, 100), proc(2, 0.0, 500), proc(3, 0.0, 300)];
        let top = top_by_memory(&procs, 2);
        assert_eq!(top[0].pid, 2);
        assert_eq!(top[1].pid, 3);
    }

    #[test]
    fn resolve_cpu_rank() {
        let procs = vec![proc(10, 1.0, 100), proc(20, 2.0, 200)];
        let cpu = top_by_cpu(&procs, 10);
        let mem = top_by_memory(&procs, 10);
        assert_eq!(resolve_selection("2", &cpu, &mem, &procs).unwrap().pid, 10);
    }

    #[test]
    fn resolve_memory_rank() {
        let procs = vec![proc(10, 1.0, 100), proc(20, 2.0, 200)];
        let cpu = top_by_cpu(&procs, 10);
        let mem = top_by_memory(&procs, 10);
        assert_eq!(resolve_selection("m1", &cpu, &mem, &procs).unwrap().pid, 20);
    }

    #[test]
    fn resolve_pid() {
        let procs = vec![proc(42, 1.0, 100)];
        let cpu = top_by_cpu(&procs, 10);
        let mem = top_by_memory(&procs, 10);
        assert_eq!(resolve_selection("42", &cpu, &mem, &procs).unwrap().pid, 42);
    }

    #[test]
    fn resolve_invalid_returns_none() {
        let procs = vec![proc(1, 1.0, 100)];
        let cpu = top_by_cpu(&procs, 10);
        let mem = top_by_memory(&procs, 10);
        assert!(resolve_selection("q", &cpu, &mem, &procs).is_none());
        assert!(resolve_selection("m99", &cpu, &mem, &procs).is_none());
    }
}
