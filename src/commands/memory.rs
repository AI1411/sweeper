use crate::commands::confirm::confirm;
use crate::json_output::{emit_json, MemoryJson, ReclaimJson, ReclaimResultJson};
use crate::memory::{
    collect_memory_report, collect_memory_report_from, docker_available, estimate_reclaim,
    execute_reclaim, format_bytes, format_estimate, format_reclaim_analysis, format_reclaim_result,
    sort_containers, LiveReclaimBackend, MemoryReport, MemorySort, POSSIBLE_CAUSES,
};
use crate::style;

pub fn run_memory(
    action: Option<crate::cli::MemoryAction>,
    sort: MemorySort,
    dry_run: bool,
    json: bool,
) -> anyhow::Result<()> {
    match action {
        Some(crate::cli::MemoryAction::Reclaim) => run_memory_reclaim(dry_run, json),
        None => run_memory_show(sort, json),
    }
}

fn run_memory_show(sort: MemorySort, json: bool) -> anyhow::Result<()> {
    let mut report = collect_memory_report()?;
    sort_containers(&mut report.containers, sort);
    if json {
        return emit_json(&MemoryJson::from(&report));
    }
    print_report(&report);
    Ok(())
}

pub fn run_memory_reclaim(dry_run: bool, json: bool) -> anyhow::Result<()> {
    if !cfg!(target_os = "macos") && !docker_available() {
        anyhow::bail!(
            "sw memory reclaim requires macOS with OrbStack/Docker or a running Docker daemon"
        );
    }
    let report = collect_memory_report()?;
    let estimate = estimate_reclaim(&report)
        .ok_or_else(|| anyhow::anyhow!("OrbStack VM memory not detected; nothing to reclaim"))?;
    if estimate.reclaimable_bytes == 0 {
        anyhow::bail!("No reclaimable memory estimated");
    }

    if json {
        let proposal = ReclaimJson::proposal(&estimate, dry_run);
        if dry_run {
            return emit_json(&proposal);
        }
        if !confirm(&format!(
            "Reclaim approximately {}?",
            format_estimate(estimate.reclaimable_bytes)
        ))? {
            println!("{}", style::dim("Cancelled."));
            return Ok(());
        }
        let backend = LiveReclaimBackend;
        let (_, result) = execute_reclaim(&backend, false)?;
        let mut out = proposal;
        out.result = result.map(ReclaimResultJson::from);
        out.executed = true;
        return emit_json(&out);
    }

    print!("{}", format_reclaim_analysis(&estimate));
    if dry_run {
        println!();
        println!(
            "{}",
            style::dim("--dry-run: would drop Linux VM page/filesystem caches via Docker")
        );
        return Ok(());
    }
    println!();
    if !confirm(&format!(
        "Reclaim approximately {}?",
        format_estimate(estimate.reclaimable_bytes)
    ))? {
        println!("{}", style::dim("Cancelled."));
        return Ok(());
    }
    let backend = LiveReclaimBackend;
    let (_, result) = execute_reclaim(&backend, false)?;
    if let Some(result) = result {
        print!("{}", format_reclaim_result(&result));
    }
    Ok(())
}

pub fn format_memory_report(report: &MemoryReport) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    writeln!(out, "{}", style::header("Memory")).unwrap();
    writeln!(out, "{}", style::dim("────────────────────────────────")).unwrap();
    writeln!(out, "{}", style::header("System")).unwrap();
    writeln!(
        out,
        "{:<20} {}",
        "Total Memory",
        style::mem(format_bytes(report.system.total_bytes))
    )
    .unwrap();
    writeln!(
        out,
        "{:<20} {}",
        "Used",
        style::mem(format_bytes(report.system.used_bytes))
    )
    .unwrap();
    writeln!(
        out,
        "{:<20} {}",
        "Available",
        style::mem(format_bytes(report.system.available_bytes))
    )
    .unwrap();

    if let Some(vm) = report.orbstack_vm_bytes {
        writeln!(out).unwrap();
        writeln!(out, "{}", style::header("OrbStack")).unwrap();
        writeln!(
            out,
            "{:<20} {}",
            "OrbStack VM",
            style::mem(format_bytes(vm))
        )
        .unwrap();
    } else if cfg!(target_os = "macos") {
        writeln!(out).unwrap();
        writeln!(out, "{}", style::header("OrbStack")).unwrap();
        if docker_available() {
            writeln!(
                out,
                "{}",
                style::dim("OrbStack VM process not detected (containers still listed below).")
            )
            .unwrap();
        } else {
            writeln!(
                out,
                "{}",
                style::dim("OrbStack/Docker not detected on this system.")
            )
            .unwrap();
        }
    } else if docker_available() {
        writeln!(out).unwrap();
        writeln!(
            out,
            "{}",
            style::dim("OrbStack VM metrics are macOS-only; showing Docker containers.")
        )
        .unwrap();
    }

    if !report.containers.is_empty() {
        writeln!(out).unwrap();
        writeln!(out, "{}", style::header("Containers")).unwrap();
        for c in &report.containers {
            writeln!(
                out,
                "{:<20} {}",
                style::process_name(&c.name),
                style::mem(format_bytes(c.memory_bytes))
            )
            .unwrap();
        }
        writeln!(out, "{}", style::dim("────────────────────────────────")).unwrap();
        writeln!(
            out,
            "{:<20} {}",
            "Container Total",
            style::mem(format_bytes(report.container_total_bytes))
        )
        .unwrap();
        if let Some(unattributed) = report.unattributed_bytes {
            if unattributed > 0 {
                writeln!(
                    out,
                    "{:<20} {}",
                    "Unattributed",
                    style::warn(format_bytes(unattributed))
                )
                .unwrap();
            }
        }
        if report.show_unattributed_warning {
            writeln!(out).unwrap();
            writeln!(
                out,
                "{}",
                style::warn("⚠ Large amount of memory is not attributed to running containers.")
            )
            .unwrap();
            writeln!(out, "Possible causes:").unwrap();
            for cause in POSSIBLE_CAUSES {
                writeln!(out, "• {cause}").unwrap();
            }
        }
    } else if docker_available() {
        writeln!(out).unwrap();
        writeln!(out, "{}", style::dim("No running Docker containers.")).unwrap();
    }

    out
}

fn print_report(report: &MemoryReport) {
    print!("{}", format_memory_report(report));
}

/// Build a report from injected data (tests / future backends).
pub fn report_from(
    system: crate::memory::SystemMemorySnapshot,
    vm_bytes: Option<u64>,
    stats_output: &str,
    ps_output: &str,
    sort: MemorySort,
) -> anyhow::Result<MemoryReport> {
    let containers = crate::memory::parse_container_stats_from(stats_output, ps_output)?;
    let mut report = collect_memory_report_from(system, vm_bytes, containers);
    sort_containers(&mut report.containers, sort);
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::SystemMemorySnapshot;

    const STATS: &str =
        "postgres\t1.2GiB / 7.776GiB\nredis\t420MiB / 7.776GiB\napi\t850MiB / 7.776GiB\n";
    const PS: &str = "postgres\tUp 2 hours\nredis\tUp 2 hours\napi\tUp 2 hours\n";

    fn system() -> SystemMemorySnapshot {
        SystemMemorySnapshot {
            total_bytes: 128 * 1024 * 1024 * 1024,
            used_bytes: 42 * 1024 * 1024 * 1024,
            available_bytes: 86 * 1024 * 1024 * 1024,
        }
    }

    #[test]
    fn formatted_report_includes_sections() {
        let report = report_from(
            system(),
            Some(18_400_000_000),
            STATS,
            PS,
            MemorySort::Memory,
        )
        .unwrap();
        let text = format_memory_report(&report);
        assert!(text.contains("System"));
        assert!(text.contains("OrbStack VM"));
        assert!(text.contains("postgres"));
        assert!(text.contains("Unattributed"));
        assert!(text.contains("Possible causes"));
    }

    #[test]
    fn json_roundtrip_fields() {
        let report = report_from(
            system(),
            Some(18_400_000_000),
            STATS,
            PS,
            MemorySort::Memory,
        )
        .unwrap();
        let json = MemoryJson::from(&report);
        assert_eq!(json.containers.len(), 3);
        assert!(json.unattributed_bytes.unwrap() > 0);
        assert!(json.show_unattributed_warning);
    }
}
