use crate::history::{append_entry, HistoryEntry, KillSignal};
use crate::process::kill::{kill_pid, KillOutcome};
use crate::process::list::find_by_name_fuzzy;

use super::confirm::confirm;

pub fn run_name(query: &str, force: bool) -> anyhow::Result<()> {
    let matches = find_by_name_fuzzy(query);
    if matches.is_empty() {
        println!("No processes matching '{query}'");
        return Ok(());
    }
    println!("Found {} processes\n", matches.len());
    for p in &matches {
        println!(
            "  {:>6}  {}  {:.1}%  {:.0} MB",
            p.pid,
            p.name,
            p.cpu,
            p.memory_mb()
        );
    }
    let total: u64 = matches.iter().map(|p| p.memory_bytes).sum();
    println!("\nTotal memory: {:.1} GB", total as f64 / 1e9);
    if !confirm("Kill all?")? {
        println!("Cancelled.");
        return Ok(());
    }
    for p in matches {
        let mut use_force = force;
        let mut outcome = kill_pid(p.pid, &p.name, use_force)?;
        if outcome == KillOutcome::StillAlive && !use_force {
            if confirm(&format!("PID {} still alive. Force kill?", p.pid))? {
                use_force = true;
                outcome = kill_pid(p.pid, &p.name, true)?;
            }
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
        println!("{} pid {}: {:?}", p.name, p.pid, outcome);
    }
    Ok(())
}
