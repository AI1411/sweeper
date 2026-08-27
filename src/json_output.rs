use serde::Serialize;

pub fn prepare_json_mode() {
    std::env::set_var("NO_COLOR", "1");
}

pub fn emit_json<T: Serialize>(value: &T) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct PortRow {
    pub port: u16,
    pub pid: u32,
    pub process: String,
}

#[derive(Debug, Serialize)]
pub struct CleanCandidateJson {
    pub pid: u32,
    pub name: String,
    pub ports: Vec<u16>,
    pub memory_bytes: u64,
    pub reasons: Vec<String>,
    pub confidence: String,
}

#[derive(Debug, Serialize)]
pub struct CleanSummaryJson {
    pub stale_servers: usize,
    pub orphans: usize,
    pub zombies: usize,
    pub idle_listeners: usize,
    pub listening: usize,
    pub estimated_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct CleanOrbstackReclaimJson {
    pub estimate: Option<ReclaimEstimateJson>,
    pub executed: bool,
    pub result: Option<ReclaimResultJson>,
}

#[derive(Debug, Serialize)]
pub struct CleanJson {
    pub candidates: Vec<CleanCandidateJson>,
    pub summary: CleanSummaryJson,
    pub orbstack_reclaim: Option<CleanOrbstackReclaimJson>,
    pub disk_reclaimable_bytes: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct ProjectJson {
    pub name: String,
    pub path: String,
    pub process_count: usize,
    pub memory_bytes: u64,
    pub ports: Vec<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_root: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compose_project: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dev_script: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HistoryJson {
    pub time: String,
    pub pid: u32,
    pub name: String,
    pub ports: Vec<u16>,
    pub signal: String,
    pub result: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SystemMemoryJson {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct ContainerMemoryJson {
    pub name: String,
    pub memory_bytes: u64,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct MemoryWarningJson {
    pub container: String,
    pub memory_bytes: u64,
    pub kind: String,
}

#[derive(Debug, Serialize)]
pub struct LeakCandidateJson {
    pub container: String,
    pub start_bytes: u64,
    pub current_bytes: u64,
    pub growth_bytes: u64,
    pub elapsed_secs: u64,
    pub start_label: String,
}

#[derive(Debug, Serialize)]
pub struct MemoryJson {
    pub system: SystemMemoryJson,
    pub orbstack_vm_bytes: Option<u64>,
    pub containers: Vec<ContainerMemoryJson>,
    pub container_total_bytes: u64,
    pub unattributed_bytes: Option<u64>,
    pub show_unattributed_warning: bool,
    pub possible_causes: Vec<&'static str>,
    pub warn_threshold_bytes: u64,
    pub warnings: Vec<MemoryWarningJson>,
    pub leak_candidates: Vec<LeakCandidateJson>,
}

impl MemoryJson {
    pub fn from_report(
        report: &crate::memory::MemoryReport,
        warnings: &[crate::memory::MemoryWarning],
        warn_threshold_bytes: u64,
        leaks: &[crate::memory::LeakCandidate],
    ) -> Self {
        Self {
            system: SystemMemoryJson {
                total_bytes: report.system.total_bytes,
                used_bytes: report.system.used_bytes,
                available_bytes: report.system.available_bytes,
            },
            orbstack_vm_bytes: report.orbstack_vm_bytes,
            containers: report
                .containers
                .iter()
                .map(|c| ContainerMemoryJson {
                    name: c.name.clone(),
                    memory_bytes: c.memory_bytes,
                    status: c.status.clone(),
                })
                .collect(),
            container_total_bytes: report.container_total_bytes,
            unattributed_bytes: report.unattributed_bytes,
            show_unattributed_warning: report.show_unattributed_warning,
            possible_causes: crate::memory::POSSIBLE_CAUSES.to_vec(),
            warn_threshold_bytes,
            warnings: warnings
                .iter()
                .map(|w| MemoryWarningJson {
                    container: w.container.clone(),
                    memory_bytes: w.memory_bytes,
                    kind: w.kind.to_string(),
                })
                .collect(),
            leak_candidates: leaks
                .iter()
                .map(|l| LeakCandidateJson {
                    container: l.container.clone(),
                    start_bytes: l.start_bytes,
                    current_bytes: l.current_bytes,
                    growth_bytes: l.growth_bytes,
                    elapsed_secs: l.elapsed_secs,
                    start_label: l.start_label.clone(),
                })
                .collect(),
        }
    }
}

impl From<&crate::memory::MemoryReport> for MemoryJson {
    fn from(report: &crate::memory::MemoryReport) -> Self {
        Self::from_report(report, &[], crate::memory::DEFAULT_WARN_BYTES, &[])
    }
}

#[derive(Debug, Serialize)]
pub struct ReclaimEstimateJson {
    pub vm_bytes: u64,
    pub container_total_bytes: u64,
    pub reclaimable_bytes: u64,
    pub page_cache_bytes: u64,
    pub filesystem_cache_bytes: u64,
    pub other_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct ReclaimResultJson {
    pub before_vm_bytes: u64,
    pub after_vm_bytes: u64,
    pub recovered_bytes: u64,
    pub success: bool,
}

#[derive(Debug, Serialize)]
pub struct ReclaimJson {
    pub estimate: ReclaimEstimateJson,
    pub dry_run: bool,
    pub executed: bool,
    pub result: Option<ReclaimResultJson>,
}

impl ReclaimJson {
    pub fn proposal(estimate: &crate::memory::ReclaimEstimate, dry_run: bool) -> Self {
        Self {
            estimate: ReclaimEstimateJson::from(estimate),
            dry_run,
            executed: false,
            result: None,
        }
    }
}

impl From<&crate::memory::ReclaimEstimate> for ReclaimEstimateJson {
    fn from(e: &crate::memory::ReclaimEstimate) -> Self {
        Self {
            vm_bytes: e.vm_bytes,
            container_total_bytes: e.container_total_bytes,
            reclaimable_bytes: e.reclaimable_bytes,
            page_cache_bytes: e.page_cache_bytes,
            filesystem_cache_bytes: e.filesystem_cache_bytes,
            other_bytes: e.other_bytes,
        }
    }
}

impl From<crate::memory::ReclaimResult> for ReclaimResultJson {
    fn from(r: crate::memory::ReclaimResult) -> Self {
        Self {
            before_vm_bytes: r.before_vm_bytes,
            after_vm_bytes: r.after_vm_bytes,
            recovered_bytes: r.recovered_bytes,
            success: r.success,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DiskRowJson {
    pub kind: String,
    pub total_bytes: u64,
    pub reclaimable_bytes: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct DiskJson {
    pub rows: Vec<DiskRowJson>,
    pub total_bytes: u64,
    pub reclaimable_bytes: u64,
}

impl From<&crate::disk::DockerDiskReport> for DiskJson {
    fn from(report: &crate::disk::DockerDiskReport) -> Self {
        Self {
            rows: report
                .rows
                .iter()
                .map(|r| DiskRowJson {
                    kind: r.kind.clone(),
                    total_bytes: r.total_bytes,
                    reclaimable_bytes: r.reclaimable_bytes,
                })
                .collect(),
            total_bytes: report.total_bytes,
            reclaimable_bytes: report.reclaimable_bytes,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DockerOverviewJson {
    pub orbstack_vm_bytes: Option<u64>,
    pub container_total_bytes: u64,
    pub memory_reclaimable_bytes: Option<u64>,
    pub disk: DiskJson,
    pub disk_reclaimable_bytes: u64,
}

impl DockerOverviewJson {
    pub fn from(
        memory: &crate::memory::MemoryReport,
        reclaim: Option<&crate::memory::ReclaimEstimate>,
        disk: &crate::disk::DockerDiskReport,
    ) -> Self {
        Self {
            orbstack_vm_bytes: memory.orbstack_vm_bytes,
            container_total_bytes: memory.container_total_bytes,
            memory_reclaimable_bytes: reclaim.map(|r| r.reclaimable_bytes),
            disk: DiskJson::from(disk),
            disk_reclaimable_bytes: disk.reclaimable_bytes,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CacheEntryJson {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub approximate: bool,
}

#[derive(Debug, Serialize)]
pub struct CacheJson {
    pub entries: Vec<CacheEntryJson>,
    pub total_bytes: u64,
}

impl CacheJson {
    pub fn from_entries(entries: &[crate::cache::CacheEntry]) -> Self {
        let total_bytes = entries.iter().map(|e| e.size_bytes).sum();
        Self {
            entries: entries
                .iter()
                .map(|e| CacheEntryJson {
                    name: e.name.clone(),
                    path: e.path.display().to_string(),
                    size_bytes: e.size_bytes,
                    approximate: e.approximate,
                })
                .collect(),
            total_bytes,
        }
    }
}
