use std::collections::BTreeMap;

use crate::history::{append_entry, entry_for_process, KillSignal};
use crate::process::kill::{kill_pid, KillOutcome};
use crate::process::list::list_processes;
use crate::process::plan::{plan_kills, print_dry_run};
use crate::process::ports::{pids_for_port, pids_for_ports};
use crate::process::tree::collect_tree_pids;
use crate::process::ProcessInfo;
use crate::style;

use super::confirm::confirm;
use super::select::{confirm_or_pick, format_process_header, format_process_row, ParseHigh};

#[derive(Debug, Clone)]
struct PortTarget {
    pid: u32,
    ports: Vec<u16>,
    info: Option<ProcessInfo>,
}

fn target_as_process(t: &PortTarget) -> ProcessInfo {
    let mut p = t.info.clone().unwrap_or_else(|| ProcessInfo {
        pid: t.pid,
        ppid: 0,
        name: "?".into(),
        cpu: 0.0,
        memory_bytes: 0,
        ports: t.ports.clone(),
        command: None,
        cwd: None,
        run_time_secs: 0,
        is_zombie: false,
    });
    p.pid = t.pid;
    p.ports.clone_from(&t.ports);
    p
}

/// Deduplicate PIDs across port bindings, merging the port list per PID.
pub fn merge_port_bindings(rows: &[(u16, u32)]) -> BTreeMap<u32, Vec<u16>> {
    let mut map: BTreeMap<u32, Vec<u16>> = BTreeMap::new();
    for &(port, pid) in rows {
        let e = map.entry(pid).or_default();
        if !e.contains(&port) {
            e.push(port);
        }
    }
    map
}

fn collect_unique_targets(
    procs: &[ProcessInfo],
    ports: &[u16],
) -> anyhow::Result<(Vec<PortTarget>, Vec<u16>)> {
    let mut rows = Vec::new();
    let mut unused = Vec::new();
    if ports.len() <= 1 {
        for &port in ports {
            let pids = pids_for_port(port)?;
            if pids.is_empty() {
                unused.push(port);
                continue;
            }
            for pid in pids {
                rows.push((port, pid));
            }
        }
    } else {
        let map = pids_for_ports(ports)?;
        for &port in ports {
            let pids = map.get(&port).cloned().unwrap_or_default();
            if pids.is_empty() {
                unused.push(port);
                continue;
            }
            for pid in pids {
                rows.push((port, pid));
            }
        }
    }
    let merged = merge_port_bindings(&rows);
    let targets = merged
        .into_iter()
        .map(|(pid, ports)| PortTarget {
            pid,
            ports,
            info: procs.iter().find(|p| p.pid == pid).cloned(),
        })
        .collect();
    Ok((targets, unused))
}

pub fn run_ports(ports: &[u16], force: bool, tree: bool, dry_run: bool) -> anyhow::Result<()> {
    let procs = list_processes();
    let (targets, unused) = collect_unique_targets(&procs, ports)?;

    for port in &unused {
        println!(
            "{} {}: {}",
            style::header("PORT"),
            style::port(port),
            style::warn("not in use")
        );
    }

    if targets.is_empty() {
        if unused.len() == ports.len() {
            println!("{}", style::warn("No listening processes found."));
        }
        return Ok(());
    }

    println!("{}", format_process_header());
    for (i, t) in targets.iter().enumerate() {
        println!("{}", format_process_row(i + 1, &target_as_process(t)));
    }

    let root_pids: Vec<u32> = targets.iter().map(|t| t.pid).collect();

    if dry_run {
        let planned = plan_kills(&procs, &root_pids, tree);
        print_dry_run(&planned, tree);
        return Ok(());
    }

    let label = if targets.len() == 1 {
        "Kill this process?".to_string()
    } else if tree {
        format!("Kill all {} process trees?", targets.len())
    } else {
        format!("Kill all {} processes?", targets.len())
    };
    let chosen = confirm_or_pick(targets.len(), &label, ParseHigh::None)?;
    if chosen.is_empty() {
        println!("{}", style::warn("Cancelled."));
        return Ok(());
    }
    let root_pids: Vec<u32> = chosen.iter().map(|&i| targets[i].pid).collect();
    let kill_pids = if tree {
        collect_tree_pids(&procs, &root_pids)
    } else {
        root_pids
    };

    let mut outcomes = Vec::new();
    for pid in kill_pids {
        let target = targets.iter().find(|t| t.pid == pid);
        let info = procs.iter().find(|p| p.pid == pid);
        let name = info
            .map(|p| p.name.as_str())
            .or_else(|| target.and_then(|t| t.info.as_ref().map(|p| p.name.as_str())))
            .unwrap_or("?");
        let mem = info
            .map(|p| p.memory_bytes)
            .or_else(|| target.and_then(|t| t.info.as_ref().map(|p| p.memory_bytes)))
            .unwrap_or(0);
        let ports_rec = target
            .map(|t| t.ports.clone())
            .or_else(|| info.map(|p| p.ports.clone()))
            .unwrap_or_default();

        let mut use_force = force;
        let mut outcome = kill_pid(pid, name, use_force)?;
        if outcome == KillOutcome::StillAlive && !use_force && confirm("Force kill?")? {
            use_force = true;
            outcome = kill_pid(pid, name, true)?;
        }
        let signal = if use_force && matches!(outcome, KillOutcome::ForceKilled) {
            KillSignal::Kill
        } else {
            KillSignal::Term
        };
        let info = procs.iter().find(|p| p.pid == pid);
        let _ = append_entry(entry_for_process(
            pid,
            name,
            ports_rec.clone(),
            signal,
            format!("{outcome:?}"),
            info,
        ));
        println!(
            "{} {} {}: {}",
            style::process_name(name),
            style::dim("pid"),
            style::pid(pid),
            style::kill_outcome(outcome)
        );
        outcomes.push(crate::report::KillResult::new(mem, ports_rec, outcome));
    }
    crate::report::print_kill_summary(&outcomes);
    Ok(())
}
