use crate::clean::{apply_excludes, excludes_from_env, propose_leftovers};
use crate::commands::confirm::confirm;
use crate::history::{append_entry, HistoryEntry, KillSignal};
use crate::process::kill::{kill_pid, KillOutcome};
use crate::process::list::list_processes;
use crate::process::ports::{listening_ports, merge_ports};
use crate::style;

pub fn run_clean(force: bool, exclude: &[String]) -> anyhow::Result<()> {
    let mut procs = list_processes();
    let ports = listening_ports().unwrap_or_default();
    merge_ports(&mut procs, &ports);
    let mut proposals = propose_leftovers(&procs, &ports);
    let mut excludes = excludes_from_env();
    excludes.extend(exclude.iter().cloned());
    proposals = apply_excludes(proposals, &excludes);

    println!("{}\n", style::header("Sweeper found possible leftovers:"));
    println!(
        "{} {} candidate processes",
        style::success("✓"),
        style::process_name(proposals.len())
    );
    for c in &proposals {
        let p = &c.process;
        println!(
            "  {} {} {} {} {:?}  {} {}",
            style::process_name(&p.name),
            style::dim("pid"),
            style::pid(p.pid),
            style::dim("ports"),
            p.ports,
            style::dim("reasons:"),
            style::warn(c.reasons.join(", "))
        );
    }
    if proposals.is_empty() {
        return Ok(());
    }
    if !confirm("Select processes to clean (confirm each)?")? {
        println!("{}", style::warn("Cancelled."));
        return Ok(());
    }
    for c in proposals {
        let p = c.process;
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
        println!(
            "{} {} {}: {}",
            style::process_name(&p.name),
            style::dim("pid"),
            style::pid(p.pid),
            style::kill_outcome(outcome)
        );
    }
    Ok(())
}
