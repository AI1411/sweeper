use crate::disk::{collect_docker_disk_report, docker_disk_available};
use crate::json_output::{emit_json, DockerOverviewJson};
use crate::memory::{
    collect_memory_report, estimate_reclaim, format_bytes, format_estimate, MemoryReport,
};
use crate::style;

pub fn run_docker(json: bool) -> anyhow::Result<()> {
    if !docker_disk_available() {
        anyhow::bail!("Docker is not available");
    }
    let memory = collect_memory_report()?;
    let reclaim = estimate_reclaim(&memory);
    let disk = collect_docker_disk_report()?;
    if json {
        return emit_json(&DockerOverviewJson::from(&memory, reclaim.as_ref(), &disk));
    }
    print_overview(&memory, reclaim.as_ref(), &disk);
    Ok(())
}

pub fn format_docker_overview(
    memory: &MemoryReport,
    reclaim: Option<&crate::memory::ReclaimEstimate>,
    disk: &crate::disk::DockerDiskReport,
) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    writeln!(out, "{}", style::header("Docker / OrbStack")).unwrap();
    writeln!(out, "{}", style::dim("────────────────────────────────")).unwrap();
    writeln!(out, "{}", style::header("Memory")).unwrap();
    if let Some(vm) = memory.orbstack_vm_bytes {
        writeln!(
            out,
            "{:<20} {}",
            "OrbStack VM",
            style::mem(format_bytes(vm))
        )
        .unwrap();
    }
    writeln!(
        out,
        "{:<20} {}",
        "Containers",
        style::mem(format_bytes(memory.container_total_bytes))
    )
    .unwrap();
    if let Some(est) = reclaim {
        writeln!(
            out,
            "{:<20} {}",
            "Reclaimable",
            style::mem(format_estimate(est.reclaimable_bytes))
        )
        .unwrap();
    }
    writeln!(out).unwrap();
    writeln!(out, "{}", style::header("Disk")).unwrap();
    for row in &disk.rows {
        writeln!(
            out,
            "{:<20} {}",
            style::process_name(&row.kind),
            style::mem(format_bytes(row.total_bytes))
        )
        .unwrap();
    }
    writeln!(out).unwrap();
    writeln!(out, "{}", style::header("Potential Recovery")).unwrap();
    writeln!(out, "{}", style::dim("────────────────────────────────")).unwrap();
    if let Some(est) = reclaim {
        writeln!(
            out,
            "{:<20} {}",
            "Memory",
            style::mem(format_estimate(est.reclaimable_bytes))
        )
        .unwrap();
    }
    writeln!(
        out,
        "{:<20} {}",
        "Disk",
        style::mem(format_bytes(disk.reclaimable_bytes))
    )
    .unwrap();
    out
}

fn print_overview(
    memory: &MemoryReport,
    reclaim: Option<&crate::memory::ReclaimEstimate>,
    disk: &crate::disk::DockerDiskReport,
) {
    print!("{}", format_docker_overview(memory, reclaim, disk));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::disk::parse_docker_system_df;
    use crate::memory::{collect_memory_report_from, ContainerStat, SystemMemorySnapshot};

    const DISK: &str = "\
TYPE            TOTAL     ACTIVE    SIZE      RECLAIMABLE
Images          10        5         8.4GB     3.1GB (37%)
Build Cache     0         0         21.4GB    18.2GB
";

    #[test]
    fn overview_includes_memory_and_disk_sections() {
        let memory = collect_memory_report_from(
            SystemMemorySnapshot {
                total_bytes: 1,
                used_bytes: 1,
                available_bytes: 1,
            },
            Some(18_400_000_000),
            vec![ContainerStat {
                name: "api".into(),
                memory_bytes: 2_500_000_000,
                status: "running".into(),
            }],
        );
        let reclaim = estimate_reclaim(&memory).unwrap();
        let disk = parse_docker_system_df(DISK).unwrap();
        let text = format_docker_overview(&memory, Some(&reclaim), &disk);
        assert!(text.contains("Docker / OrbStack"));
        assert!(text.contains("Potential Recovery"));
        assert!(text.contains("Build Cache"));
    }
}
