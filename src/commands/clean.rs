use crate::clean::{
    apply_excludes, confidence_level, excludes_from_env, format_age, format_command_snippet,
    format_reasons_display, propose_leftovers, summarize, CleanSummary,
};
use crate::commands::confirm::confirm;
use crate::history::{append_entry, HistoryEntry, KillSignal};
use crate::process::kill::{kill_pid, KillOutcome};
use crate::process::list::list_processes;
use crate::process::ports::{listening_ports, merge_ports};
use crate::style;
use crate::style as sty;

pub fn run_clean(force: bool, exclude: &[String]) -> anyhow::Result<()> {
    let mut procs = list_processes();
    let ports = listening_ports().unwrap_or_default();
    merge_ports(&mut procs, &ports);
    let mut proposals = propose_leftovers(&procs, &ports);
    let mut excludes = excludes_from_env();
    excludes.extend(exclude.iter().cloned());
    proposals = apply_excludes(proposals, &excludes);

    let summary = summarize(&proposals);
    println!("{}\n", style::header("Sweeper found possible leftovers:"));
    print_summary_lines(&summary, proposals.len());

    for c in &proposals {
        let p = &c.process;
        let age = format_age(p.run_time_secs);
        let reasons = format_reasons_display(c);
        let confidence = confidence_level(c);
        println!(
            "  {} {} {} {} {:?} {} {}  {} {}  {} {}",
            style::process_name(&p.name),
            style::dim("pid"),
            style::pid(p.pid),
            style::dim("ports"),
            p.ports,
            style::dim("age"),
            style::dim(age),
            style::dim("confidence:"),
            style::warn(confidence),
            style::dim("reasons:"),
            style::warn(reasons.join(", "))
        );
        if let Some(cmd) = format_command_snippet(p.command.as_deref()) {
            println!("    {} {}", style::dim("cmd:"), style::dim(cmd));
        }
    }
    if proposals.is_empty() {
        return Ok(());
    }
    if !confirm("Select processes to clean (confirm each)?")? {
        println!("{}", style::warn("Cancelled."));
        return Ok(());
    }
    let mut outcomes = Vec::new();
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
        outcomes.push(crate::report::KillResult::new(
            p.memory_bytes,
            p.ports.clone(),
            outcome,
        ));
    }
    crate::report::print_kill_summary(&outcomes);
    Ok(())
}

fn print_summary_lines(summary: &CleanSummary, total: usize) {
    if total == 0 {
        println!("{} No leftover candidates right now.", style::dim("·"));
        return;
    }
    println!(
        "{} {} candidate process{}",
        style::success("✓"),
        style::process_name(total),
        if total == 1 { "" } else { "es" }
    );
    if summary.stale_servers > 0 {
        println!(
            "{} {} stale dev server{}",
            style::success("✓"),
            summary.stale_servers,
            if summary.stale_servers == 1 { "" } else { "s" }
        );
    }
    if summary.orphans > 0 {
        println!(
            "{} {} orphan process{}",
            style::success("✓"),
            summary.orphans,
            if summary.orphans == 1 { "" } else { "es" }
        );
    }
    if summary.zombies > 0 {
        println!(
            "{} {} zombie process{}",
            style::success("✓"),
            summary.zombies,
            if summary.zombies == 1 { "" } else { "es" }
        );
    }
    if summary.idle_listeners > 0 {
        println!(
            "{} {} idle listener{}",
            style::success("✓"),
            summary.idle_listeners,
            if summary.idle_listeners == 1 { "" } else { "s" }
        );
    }
    if summary.listening > 0 {
        println!(
            "{} {} listening on dev port{}",
            style::dim("·"),
            summary.listening,
            if summary.listening == 1 { "" } else { "s" }
        );
    }
    if summary.estimated_bytes > 0 {
        let mb = summary.estimated_bytes as f64 / (1024.0 * 1024.0);
        println!(
            "{} {}",
            style::dim("Estimated memory reclaim:"),
            sty::mem(format!("{mb:.0} MB"))
        );
    }
    println!();
}
