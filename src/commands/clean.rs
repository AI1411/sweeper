use crate::clean::{
    apply_excludes, confidence_level, excludes_from_env, format_age, format_command_snippet,
    format_reasons_display, propose_leftovers, summarize, CleanSummary,
};
use crate::commands::confirm::confirm;
use crate::disk::collect_docker_disk_report;
use crate::history::{append_entry, HistoryEntry, KillSignal};
use crate::json_output::{
    emit_json, CleanCandidateJson, CleanJson, CleanOrbstackReclaimJson, CleanSummaryJson,
    ReclaimEstimateJson, ReclaimResultJson,
};
use crate::memory::{
    collect_memory_report, estimate_reclaim, execute_reclaim, format_bytes, format_estimate,
    format_reclaim_result, LiveReclaimBackend, ReclaimEstimate, ReclaimResult,
};
use crate::process::kill::{kill_pid, KillOutcome};
use crate::process::list::list_processes;
use crate::process::plan::{plan_kills, print_dry_run};
use crate::process::ports::{listening_ports, merge_ports};
use crate::style;

pub fn run_clean(force: bool, exclude: &[String], dry_run: bool, json: bool) -> anyhow::Result<()> {
    let mut procs = list_processes();
    let ports = listening_ports().unwrap_or_default();
    merge_ports(&mut procs, &ports);
    let mut proposals = propose_leftovers(&procs, &ports);
    let mut excludes = excludes_from_env();
    excludes.extend(exclude.iter().cloned());
    proposals = apply_excludes(proposals, &excludes);

    let summary = summarize(&proposals);
    let (reclaim_estimate, disk_reclaimable) = orbstack_context();

    if json {
        let payload = build_clean_json(
            &proposals,
            &summary,
            &reclaim_estimate,
            disk_reclaimable,
            None,
        );
        return emit_json(&payload);
    }

    println!("{}\n", style::header("Sweeper found possible leftovers:"));
    print_summary_lines(&summary, proposals.len(), reclaim_estimate.as_ref());

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
    if dry_run {
        let roots: Vec<u32> = proposals.iter().map(|c| c.process.pid).collect();
        let planned = plan_kills(&procs, &roots, false);
        print_dry_run(&planned, false);
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

    let reclaim_result = try_post_clean_reclaim(reclaim_estimate.as_ref())?;
    if reclaim_result.is_some() || outcomes.iter().any(|o| o.is_success()) {
        print_recovered_summary(&outcomes, reclaim_result.as_ref(), disk_reclaimable);
    }
    Ok(())
}

fn orbstack_context() -> (Option<ReclaimEstimate>, Option<u64>) {
    let memory_report = collect_memory_report().ok();
    let estimate = memory_report.as_ref().and_then(estimate_reclaim);
    let disk_reclaimable = collect_docker_disk_report()
        .ok()
        .map(|report| report.reclaimable_bytes);
    (estimate, disk_reclaimable)
}

fn try_post_clean_reclaim(estimate: Option<&ReclaimEstimate>) -> anyhow::Result<Option<ReclaimResult>> {
    let Some(est) = estimate else {
        return Ok(None);
    };
    if est.reclaimable_bytes == 0 {
        return Ok(None);
    }
    println!();
    println!(
        "{} {}",
        style::dim("OrbStack reclaim available:"),
        style::mem(format_estimate(est.reclaimable_bytes))
    );
    if !confirm(&format!(
        "Reclaim approximately {} of OrbStack VM memory?",
        format_estimate(est.reclaimable_bytes)
    ))? {
        println!("{}", style::dim("Skipped OrbStack memory reclaim."));
        return Ok(None);
    }
    let backend = LiveReclaimBackend;
    let (_, result) = execute_reclaim(&backend, false)?;
    if let Some(result) = &result {
        print!("{}", format_reclaim_result(result));
    }
    Ok(result)
}

fn build_clean_json(
    proposals: &[crate::clean::CleanCandidate],
    summary: &CleanSummary,
    reclaim_estimate: &Option<ReclaimEstimate>,
    disk_reclaimable: Option<u64>,
    reclaim_result: Option<&ReclaimResult>,
) -> CleanJson {
    CleanJson {
        candidates: proposals
            .iter()
            .map(|c| CleanCandidateJson {
                pid: c.process.pid,
                name: c.process.name.clone(),
                ports: c.process.ports.clone(),
                memory_bytes: c.process.memory_bytes,
                reasons: format_reasons_display(c),
                confidence: confidence_level(c).into(),
            })
            .collect(),
        summary: CleanSummaryJson {
            stale_servers: summary.stale_servers,
            orphans: summary.orphans,
            zombies: summary.zombies,
            idle_listeners: summary.idle_listeners,
            listening: summary.listening,
            estimated_bytes: summary.estimated_bytes,
        },
        orbstack_reclaim: reclaim_estimate.as_ref().map(|e| CleanOrbstackReclaimJson {
            estimate: Some(ReclaimEstimateJson::from(e)),
            executed: reclaim_result.is_some(),
            result: reclaim_result.cloned().map(ReclaimResultJson::from),
        }),
        disk_reclaimable_bytes: disk_reclaimable,
    }
}

fn print_recovered_summary(
    outcomes: &[crate::report::KillResult],
    reclaim: Option<&ReclaimResult>,
    disk_reclaimable: Option<u64>,
) {
    print!("{}", format_recovered_summary(outcomes, reclaim, disk_reclaimable));
}

pub fn format_recovered_summary(
    outcomes: &[crate::report::KillResult],
    reclaim: Option<&ReclaimResult>,
    disk_reclaimable: Option<u64>,
) -> String {
    use std::fmt::Write;
    let killed = outcomes.iter().filter(|o| o.is_success()).count();
    let process_memory = crate::report::freed_bytes_from_results(outcomes);
    let orbstack_memory = reclaim.map(|r| r.recovered_bytes).unwrap_or(0);
    if killed == 0 && orbstack_memory == 0 {
        return String::new();
    }
    let mut out = String::new();
    writeln!(out).unwrap();
    writeln!(out, "{}", style::header("Recovered")).unwrap();
    if process_memory > 0 {
        writeln!(
            out,
            "{:<14} {}",
            "Processes",
            style::mem(format_bytes(process_memory))
        )
        .unwrap();
    }
    if orbstack_memory > 0 {
        writeln!(
            out,
            "{:<14} {}",
            "Memory",
            style::mem(format_estimate(orbstack_memory))
        )
        .unwrap();
    }
    if let Some(disk) = disk_reclaimable {
        if disk > 0 {
            writeln!(
                out,
                "{:<14} {} {}",
                "Disk",
                style::mem(format_estimate(disk)),
                style::dim("(informational — run sw disk for prune options)")
            )
            .unwrap();
        }
    }
    out
}

fn print_summary_lines(summary: &CleanSummary, total: usize, reclaim_estimate: Option<&ReclaimEstimate>) {
    print!("{}", format_summary_lines(summary, total, reclaim_estimate));
}

pub fn format_summary_lines(
    summary: &CleanSummary,
    total: usize,
    reclaim_estimate: Option<&ReclaimEstimate>,
) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    if total == 0 {
        writeln!(out, "{} No leftover candidates right now.", style::dim("·")).unwrap();
        if let Some(est) = reclaim_estimate {
            if est.reclaimable_bytes > 0 {
                writeln!(
                    out,
                    "{} {}",
                    style::dim("OrbStack reclaim available:"),
                    style::mem(format_estimate(est.reclaimable_bytes))
                )
                .unwrap();
            }
        }
        return out;
    }
    writeln!(
        out,
        "{} {} candidate process{}",
        style::success("✓"),
        style::process_name(total),
        if total == 1 { "" } else { "es" }
    )
    .unwrap();
    if summary.stale_servers > 0 {
        writeln!(
            out,
            "{} {} stale dev server{}",
            style::success("✓"),
            summary.stale_servers,
            if summary.stale_servers == 1 { "" } else { "s" }
        )
        .unwrap();
    }
    if summary.orphans > 0 {
        writeln!(
            out,
            "{} {} orphan process{}",
            style::success("✓"),
            summary.orphans,
            if summary.orphans == 1 { "" } else { "es" }
        )
        .unwrap();
    }
    if summary.zombies > 0 {
        writeln!(
            out,
            "{} {} zombie process{}",
            style::success("✓"),
            summary.zombies,
            if summary.zombies == 1 { "" } else { "es" }
        )
        .unwrap();
    }
    if summary.idle_listeners > 0 {
        writeln!(
            out,
            "{} {} idle listener{}",
            style::success("✓"),
            summary.idle_listeners,
            if summary.idle_listeners == 1 { "" } else { "s" }
        )
        .unwrap();
    }
    if summary.listening > 0 {
        writeln!(
            out,
            "{} {} listening on dev port{}",
            style::dim("·"),
            summary.listening,
            if summary.listening == 1 { "" } else { "s" }
        )
        .unwrap();
    }
    if summary.estimated_bytes > 0 {
        let mb = summary.estimated_bytes as f64 / (1024.0 * 1024.0);
        writeln!(
            out,
            "{} {}",
            style::dim("Estimated memory reclaim:"),
            style::mem(format!("{mb:.0} MB"))
        )
        .unwrap();
    }
    if let Some(est) = reclaim_estimate {
        if est.reclaimable_bytes > 0 {
            writeln!(
                out,
                "{} {}",
                style::dim("OrbStack reclaim available:"),
                style::mem(format_estimate(est.reclaimable_bytes))
            )
            .unwrap();
        }
    }
    writeln!(out).unwrap();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clean::CleanSummary;
    use crate::memory::ReclaimEstimate;

    fn sample_estimate() -> ReclaimEstimate {
        ReclaimEstimate {
            vm_bytes: 18_400_000_000,
            container_total_bytes: 2_500_000_000,
            reclaimable_bytes: 14_200_000_000,
            page_cache_bytes: 9_000_000_000,
            filesystem_cache_bytes: 3_000_000_000,
            other_bytes: 2_200_000_000,
        }
    }

    #[test]
    fn summary_includes_orbstack_reclaim_hint() {
        let text = format_summary_lines(&CleanSummary::default(), 2, Some(&sample_estimate()));
        assert!(text.contains("OrbStack reclaim available"));
    }

    #[test]
    fn recovered_summary_includes_memory_and_disk() {
        let outcomes = vec![crate::report::KillResult::new(
            100_000_000,
            vec![3000],
            KillOutcome::Terminated,
        )];
        let reclaim = ReclaimResult {
            before_vm_bytes: 18_000_000_000,
            after_vm_bytes: 4_000_000_000,
            recovered_bytes: 14_000_000_000,
            success: true,
        };
        let text = format_recovered_summary(&outcomes, Some(&reclaim), Some(8_700_000_000));
        assert!(text.contains("Recovered"));
        assert!(text.contains("Memory"));
        assert!(text.contains("Disk"));
    }
}
