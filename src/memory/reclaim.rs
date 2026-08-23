use std::process::Command;

use super::docker::docker_available;
use super::format::{format_bytes, format_estimate};
use super::{collect_memory_report, MemoryReport};

/// Heuristic split of unattributed VM memory into reclaimable sources.
const PAGE_CACHE_RATIO: f64 = 0.64;
const FILESYSTEM_CACHE_RATIO: f64 = 0.24;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReclaimEstimate {
    pub vm_bytes: u64,
    pub container_total_bytes: u64,
    pub reclaimable_bytes: u64,
    pub page_cache_bytes: u64,
    pub filesystem_cache_bytes: u64,
    pub other_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReclaimResult {
    pub before_vm_bytes: u64,
    pub after_vm_bytes: u64,
    pub recovered_bytes: u64,
    pub success: bool,
}

pub trait ReclaimBackend {
    fn drop_caches(&self) -> anyhow::Result<()>;
}

pub struct LiveReclaimBackend;

impl ReclaimBackend for LiveReclaimBackend {
    fn drop_caches(&self) -> anyhow::Result<()> {
        if !docker_available() {
            anyhow::bail!("Docker is not available; cannot reclaim OrbStack VM caches");
        }
        let output = Command::new("docker")
            .args([
                "run",
                "--rm",
                "--privileged",
                "alpine",
                "sh",
                "-c",
                "sync; echo 3 > /proc/sys/vm/drop_caches",
            ])
            .output()?;
        if !output.status.success() {
            anyhow::bail!(
                "cache reclaim failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(())
    }
}

/// Estimate reclaimable memory from a memory report.
pub fn estimate_reclaim(report: &MemoryReport) -> Option<ReclaimEstimate> {
    let vm_bytes = report.orbstack_vm_bytes?;
    let unattributed = report.unattributed_bytes.unwrap_or(0);
    if unattributed == 0 {
        return Some(ReclaimEstimate {
            vm_bytes,
            container_total_bytes: report.container_total_bytes,
            reclaimable_bytes: 0,
            page_cache_bytes: 0,
            filesystem_cache_bytes: 0,
            other_bytes: 0,
        });
    }
    let reclaimable_bytes = unattributed;
    let page_cache_bytes = (reclaimable_bytes as f64 * PAGE_CACHE_RATIO) as u64;
    let filesystem_cache_bytes = (reclaimable_bytes as f64 * FILESYSTEM_CACHE_RATIO) as u64;
    let other_bytes = reclaimable_bytes
        .saturating_sub(page_cache_bytes)
        .saturating_sub(filesystem_cache_bytes);
    Some(ReclaimEstimate {
        vm_bytes,
        container_total_bytes: report.container_total_bytes,
        reclaimable_bytes,
        page_cache_bytes,
        filesystem_cache_bytes,
        other_bytes,
    })
}

pub fn format_reclaim_analysis(estimate: &ReclaimEstimate) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    writeln!(out, "OrbStack Memory").unwrap();
    writeln!(out, "────────────────────────────────").unwrap();
    writeln!(
        out,
        "{:<20} {}",
        "VM Memory",
        format_bytes(estimate.vm_bytes)
    )
    .unwrap();
    writeln!(
        out,
        "{:<20} {}",
        "Containers",
        format_bytes(estimate.container_total_bytes)
    )
    .unwrap();
    writeln!(out, "Estimated").unwrap();
    writeln!(
        out,
        "{:<20} {}",
        "Reclaimable",
        format_estimate(estimate.reclaimable_bytes)
    )
    .unwrap();
    writeln!(out, "Possible sources:").unwrap();
    writeln!(
        out,
        "{:<20} {}",
        "Linux page cache",
        format_estimate(estimate.page_cache_bytes)
    )
    .unwrap();
    writeln!(
        out,
        "{:<20} {}",
        "Filesystem cache",
        format_estimate(estimate.filesystem_cache_bytes)
    )
    .unwrap();
    writeln!(
        out,
        "{:<20} {}",
        "Other",
        format_estimate(estimate.other_bytes)
    )
    .unwrap();
    out
}

pub fn format_reclaim_result(result: &ReclaimResult) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    writeln!(out, "Reclaiming memory...").unwrap();
    writeln!(
        out,
        "{:<20} {}",
        "Before",
        format_bytes(result.before_vm_bytes)
    )
    .unwrap();
    writeln!(
        out,
        "{:<20} {}",
        "After",
        format_bytes(result.after_vm_bytes)
    )
    .unwrap();
    writeln!(out, "────────────────────────────────").unwrap();
    writeln!(
        out,
        "{:<20} {}",
        "Recovered",
        format_bytes(result.recovered_bytes)
    )
    .unwrap();
    if result.success {
        writeln!(out, "✓ Memory reclaimed successfully").unwrap();
    } else {
        writeln!(out, "✗ Memory reclaim did not reduce VM usage").unwrap();
    }
    out
}

pub fn execute_reclaim<B: ReclaimBackend>(
    backend: &B,
    dry_run: bool,
) -> anyhow::Result<(ReclaimEstimate, Option<ReclaimResult>)> {
    let before = collect_memory_report()?;
    let estimate = estimate_reclaim(&before)
        .ok_or_else(|| anyhow::anyhow!("OrbStack VM memory not detected; nothing to reclaim"))?;
    if estimate.reclaimable_bytes == 0 {
        anyhow::bail!("No reclaimable memory estimated");
    }
    if dry_run {
        return Ok((estimate, None));
    }
    backend.drop_caches()?;
    let after = collect_memory_report()?;
    let after_vm = after.orbstack_vm_bytes.unwrap_or(estimate.vm_bytes);
    let before_vm = before.orbstack_vm_bytes.unwrap_or(estimate.vm_bytes);
    let recovered = before_vm.saturating_sub(after_vm);
    let success = recovered > 0;
    Ok((
        estimate,
        Some(ReclaimResult {
            before_vm_bytes: before_vm,
            after_vm_bytes: after_vm,
            recovered_bytes: recovered,
            success,
        }),
    ))
}

#[cfg(test)]
pub struct MockReclaimBackend {
    pub calls: std::sync::Mutex<usize>,
}

#[cfg(test)]
impl MockReclaimBackend {
    pub fn new() -> Self {
        Self {
            calls: std::sync::Mutex::new(0),
        }
    }
}

#[cfg(test)]
impl ReclaimBackend for MockReclaimBackend {
    fn drop_caches(&self) -> anyhow::Result<()> {
        *self.calls.lock().unwrap() += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{collect_memory_report_from, ContainerStat, SystemMemorySnapshot};

    fn report_with_gap() -> MemoryReport {
        collect_memory_report_from(
            SystemMemorySnapshot {
                total_bytes: 128 * 1024 * 1024 * 1024,
                used_bytes: 42 * 1024 * 1024 * 1024,
                available_bytes: 86 * 1024 * 1024 * 1024,
            },
            Some(18_400_000_000),
            vec![ContainerStat {
                name: "api".into(),
                memory_bytes: 2_500_000_000,
                status: "running".into(),
            }],
        )
    }

    #[test]
    fn estimate_splits_unattributed() {
        let est = estimate_reclaim(&report_with_gap()).unwrap();
        assert!(est.reclaimable_bytes > 10_000_000_000);
        assert!(est.page_cache_bytes > est.filesystem_cache_bytes);
        let sum = est.page_cache_bytes + est.filesystem_cache_bytes + est.other_bytes;
        assert_eq!(sum, est.reclaimable_bytes);
    }

    #[test]
    fn analysis_format_includes_tilde() {
        let est = estimate_reclaim(&report_with_gap()).unwrap();
        let text = format_reclaim_analysis(&est);
        assert!(text.contains("Reclaimable"));
        assert!(text.contains('~'));
    }

    #[test]
    fn mock_backend_records_call() {
        let backend = MockReclaimBackend::new();
        backend.drop_caches().unwrap();
        assert_eq!(*backend.calls.lock().unwrap(), 1);
    }
}
