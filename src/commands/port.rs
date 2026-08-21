use crate::history::{append_entry, HistoryEntry, KillSignal};
use crate::process::kill::{kill_pid, KillOutcome};
use crate::process::list::list_processes;
use crate::process::ports::pids_for_port;

use super::confirm::confirm;

pub fn run_ports(ports: &[u16], force: bool) -> anyhow::Result<()> {
    let procs = list_processes();
    for port in ports {
        let pids = pids_for_port(*port)?;
        if pids.is_empty() {
            println!("PORT {port}: not in use");
            continue;
        }
        for pid in pids {
            let info = procs.iter().find(|p| p.pid == pid);
            let name = info.map(|p| p.name.as_str()).unwrap_or("?");
            let cpu = info.map(|p| p.cpu).unwrap_or(0.0);
            let mem = info.map(|p| p.memory_mb()).unwrap_or(0.0);
            println!("PORT  PID    PROCESS     CPU    MEM");
            println!("{port:<5} {pid:<6} {name:<10} {cpu:.1}%  {mem:.0}MB");
            if !confirm("Kill this process?")? {
                continue;
            }
            let mut use_force = force;
            let mut outcome = kill_pid(pid, name, use_force)?;
            if outcome == KillOutcome::StillAlive && !use_force {
                if confirm("Force kill?")? {
                    use_force = true;
                    outcome = kill_pid(pid, name, true)?;
                }
            }
            let signal = if use_force && matches!(outcome, KillOutcome::ForceKilled) {
                KillSignal::Kill
            } else {
                KillSignal::Term
            };
            let _ = append_entry(HistoryEntry::new(
                pid,
                name,
                vec![*port],
                signal,
                format!("{outcome:?}"),
            ));
            println!("{:?}", outcome);
        }
    }
    Ok(())
}
