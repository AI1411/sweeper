use crate::history::{append_entry, HistoryEntry, KillSignal};
use crate::process::kill::{kill_pid, KillOutcome};
use crate::process::list::list_processes;
use crate::process::ports::pids_for_port;
use crate::process::tree::collect_tree_pids;
use crate::style;

use super::confirm::confirm;

pub fn run_ports(ports: &[u16], force: bool, tree: bool) -> anyhow::Result<()> {
    let procs = list_processes();
    for port in ports {
        let pids = pids_for_port(*port)?;
        if pids.is_empty() {
            println!(
                "{} {}: {}",
                style::header("PORT"),
                style::port(port),
                style::warn("not in use")
            );
            continue;
        }
        for pid in pids {
            let info = procs.iter().find(|p| p.pid == pid);
            let name = info.map(|p| p.name.as_str()).unwrap_or("?");
            let cpu = info.map(|p| p.cpu).unwrap_or(0.0);
            let mem = info.map(|p| p.memory_mb()).unwrap_or(0.0);
            println!(
                "{}  {}    {}     {}    {}",
                style::header("PORT"),
                style::header("PID"),
                style::header("PROCESS"),
                style::header("CPU"),
                style::header("MEM")
            );
            println!(
                "{:<5} {} {} {}  {}",
                style::port(format!("{port:<5}")),
                style::pid(format!("{pid:<6}")),
                style::process_name(format!("{name:<10}")),
                style::cpu(cpu),
                style::mem(format!("{mem:.0}MB"))
            );

            let kill_pids = if tree {
                collect_tree_pids(&procs, &[pid])
            } else {
                vec![pid]
            };
            if tree && kill_pids.len() > 1 {
                println!(
                    "{}",
                    style::dim(format!("  tree: {} processes", kill_pids.len()))
                );
            }

            if !confirm(if tree {
                "Kill this process tree?"
            } else {
                "Kill this process?"
            })? {
                continue;
            }

            for kid in kill_pids {
                let (kname, kports) = procs
                    .iter()
                    .find(|p| p.pid == kid)
                    .map(|p| (p.name.as_str(), p.ports.clone()))
                    .unwrap_or_else(|| {
                        if kid == pid {
                            (name, vec![*port])
                        } else {
                            ("?", vec![])
                        }
                    });
                let mut use_force = force;
                let mut outcome = kill_pid(kid, kname, use_force)?;
                if outcome == KillOutcome::StillAlive && !use_force && confirm("Force kill?")? {
                    use_force = true;
                    outcome = kill_pid(kid, kname, true)?;
                }
                let signal = if use_force && matches!(outcome, KillOutcome::ForceKilled) {
                    KillSignal::Kill
                } else {
                    KillSignal::Term
                };
                let ports_rec = if kports.is_empty() && kid == pid {
                    vec![*port]
                } else {
                    kports
                };
                let _ = append_entry(HistoryEntry::new(
                    kid,
                    kname,
                    ports_rec,
                    signal,
                    format!("{outcome:?}"),
                ));
                println!(
                    "{} {} {}: {}",
                    style::process_name(kname),
                    style::dim("pid"),
                    style::pid(kid),
                    style::kill_outcome(outcome)
                );
            }
        }
    }
    Ok(())
}
