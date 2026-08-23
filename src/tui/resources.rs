use crate::disk::{collect_docker_disk_report, docker_disk_available, DockerDiskReport};
use crate::memory::{
    collect_memory_report, estimate_reclaim, format_bytes, format_estimate, ContainerMemory,
    MemoryReport, ReclaimEstimate,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResourcePanel {
    #[default]
    Summary,
    Containers,
    Docker,
}

#[derive(Debug, Clone, Default)]
pub struct ResourceSnapshot {
    pub available: bool,
    pub orbstack_vm_bytes: Option<u64>,
    pub memory_reclaimable: Option<u64>,
    pub container_total_bytes: u64,
    pub containers: Vec<ContainerMemory>,
    pub disk_total_bytes: Option<u64>,
    pub disk_reclaimable: Option<u64>,
    pub reclaim_estimate: Option<ReclaimEstimate>,
    pub memory_report: Option<MemoryReport>,
    pub disk_report: Option<DockerDiskReport>,
}

impl ResourceSnapshot {
    pub fn unavailable() -> Self {
        Self {
            available: false,
            ..Default::default()
        }
    }

    pub fn summary_line(&self) -> Option<String> {
        if !self.available {
            return None;
        }
        let vm = self
            .orbstack_vm_bytes
            .map(format_bytes)
            .unwrap_or_else(|| "—".into());
        let reclaim = self
            .memory_reclaimable
            .map(format_estimate)
            .unwrap_or_else(|| "—".into());
        Some(format!("OrbStack {vm}  ({reclaim} reclaim)"))
    }

    pub fn disk_summary_line(&self) -> Option<String> {
        if !self.available {
            return None;
        }
        let total = self
            .disk_total_bytes
            .map(format_bytes)
            .unwrap_or_else(|| "—".into());
        let reclaim = self
            .disk_reclaimable
            .map(format_bytes)
            .unwrap_or_else(|| "—".into());
        Some(format!("Docker disk {total}  ({reclaim} recoverable)"))
    }
}

pub fn load_resource_snapshot() -> ResourceSnapshot {
    if !docker_disk_available() {
        return ResourceSnapshot::unavailable();
    }
    let memory = collect_memory_report().ok();
    let reclaim = memory.as_ref().and_then(estimate_reclaim);
    let disk = collect_docker_disk_report().ok();
    ResourceSnapshot {
        available: true,
        orbstack_vm_bytes: memory.as_ref().and_then(|m| m.orbstack_vm_bytes),
        memory_reclaimable: reclaim.as_ref().map(|r| r.reclaimable_bytes),
        container_total_bytes: memory
            .as_ref()
            .map(|m| m.container_total_bytes)
            .unwrap_or(0),
        containers: memory
            .as_ref()
            .map(|m| m.containers.clone())
            .unwrap_or_default(),
        disk_total_bytes: disk.as_ref().map(|d| d.total_bytes),
        disk_reclaimable: disk.as_ref().map(|d| d.reclaimable_bytes),
        reclaim_estimate: reclaim,
        memory_report: memory,
        disk_report: disk,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_snapshot_has_no_summary() {
        let snap = ResourceSnapshot::unavailable();
        assert!(snap.summary_line().is_none());
    }
}
