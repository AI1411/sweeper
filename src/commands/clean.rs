use crate::clean::propose_leftovers;
use crate::commands::confirm::confirm;
use crate::history::{append_entry, HistoryEntry, KillSignal};
use crate::process::kill::{kill_pid, KillOutcome};
use crate::process::list::list_processes;
use crate::process::ports::{listening_ports, merge_ports};

pub fn run_clean(force: bool) -> anyhow::Result<()> {
    let mut procs = list_processes();
    let ports = listening_ports().unwrap_or_default();
    merge_ports(&mut procs, &ports);
    let proposals = propose_leftovers(&procs, &ports);
    println!("Sweeper found possible leftovers:\n");
    println!("✓ {} candidate processes", proposals.len());
    for p in &proposals {
        println!("  {} pid {} ports {:?}", p.name, p.pid, p.ports);
    }
    if proposals.is_empty() {
        return Ok(());
    }
    if !confirm("Select processes to clean (confirm each)?")? {
        println!("Cancelled.");
        return Ok(());
    }
    for p in proposals {
        if !confirm(&format!("Kill {} (pid {})?", p.name, p.pid))? {
            continue;
        }
        let mut use_force = force;
        let mut outcome = kill_pid(p.pid, &p.name, use_force)?;
        if outcome == KillOutcome::StillAlive && !use_force && confirm("Force kill?")? {
            use_force = true;
            outcome = kill_pid(p.pid, &p.name, true)?;
        }
        let signal = if use_force && matches!(outcome, KillOutcome::ForceKilled) {
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
        println!("{} pid {}: {:?}", p.name, p.pid, outcome);
    }
    Ok(())
}
