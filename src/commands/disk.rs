use crate::disk::{collect_docker_disk_report, docker_disk_available, DockerDiskReport};
use crate::json_output::{emit_json, DiskJson};
use crate::memory::format_bytes;
use crate::style;

pub fn run_disk(top: Option<usize>, json: bool) -> anyhow::Result<()> {
    if !docker_disk_available() {
        anyhow::bail!("Docker is not available");
    }
    let report = collect_docker_disk_report()?;
    if json {
        return emit_json(&DiskJson::from(&report));
    }
    print_disk_report(&report, top);
    Ok(())
}

pub fn format_disk_report(report: &DockerDiskReport, top: Option<usize>) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    writeln!(out, "{}", style::header("Disk (Docker)")).unwrap();
    writeln!(out, "{}", style::dim("────────────────────────────────")).unwrap();
    let rows: Vec<_> = if let Some(n) = top {
        report.rows.iter().take(n).collect()
    } else {
        report.rows.iter().collect()
    };
    for row in rows {
        let reclaim = row
            .reclaimable_bytes
            .map(|b| format!("(reclaimable {})", format_bytes(b)))
            .unwrap_or_default();
        writeln!(
            out,
            "{:<16} {} {}",
            style::process_name(&row.kind),
            style::mem(format_bytes(row.total_bytes)),
            style::dim(reclaim)
        )
        .unwrap();
    }
    writeln!(out, "{}", style::dim("────────────────────────────────")).unwrap();
    writeln!(
        out,
        "{:<16} {}",
        "Total",
        style::mem(format_bytes(report.total_bytes))
    )
    .unwrap();
    writeln!(
        out,
        "{:<16} {}",
        "Reclaimable",
        style::mem(format_bytes(report.reclaimable_bytes))
    )
    .unwrap();
    out
}

fn print_disk_report(report: &DockerDiskReport, top: Option<usize>) {
    print!("{}", format_disk_report(report, top));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::disk::parse_docker_system_df;

    const FIXTURE: &str = "\
TYPE            TOTAL     ACTIVE    SIZE      RECLAIMABLE
Images          10        5         8.4GB     3.1GB (37%)
Containers      5         2         1.2GB     800MB (66%)
";

    #[test]
    fn formatted_report_lists_kinds() {
        let report = parse_docker_system_df(FIXTURE).unwrap();
        let text = format_disk_report(&report, None);
        assert!(text.contains("Images"));
        assert!(text.contains("Reclaimable"));
    }
}
