mod docker;
mod format;
mod orbstack;
mod reclaim;
mod system;
mod watch;
mod warnings;

pub use watch::{compute_delta, collect_watch_sample, run_memory_watch, WatchDelta, WatchSample};

pub use warnings::{
    high_memory_warnings, parse_warn_threshold, warn_threshold_bytes, MemoryWarning,
    DEFAULT_WARN_BYTES,
};

pub use docker::{
    docker_available, parse_container_stats, parse_container_stats_from, ContainerStat,
};
pub use format::{format_bytes, format_estimate, parse_docker_bytes};
pub use orbstack::orbstack_vm_bytes;
pub use reclaim::{
    estimate_reclaim, execute_reclaim, format_reclaim_analysis, format_reclaim_result,
    LiveReclaimBackend, ReclaimBackend, ReclaimEstimate, ReclaimResult,
};
pub use system::system_memory;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MemorySort {
    #[default]
    Memory,
    Name,
    Status,
}

impl MemorySort {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "memory" | "mem" => Some(Self::Memory),
            "name" => Some(Self::Name),
            "status" => Some(Self::Status),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemMemorySnapshot {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerMemory {
    pub name: String,
    pub memory_bytes: u64,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryReport {
    pub system: SystemMemorySnapshot,
    pub orbstack_vm_bytes: Option<u64>,
    pub containers: Vec<ContainerMemory>,
    pub container_total_bytes: u64,
    pub unattributed_bytes: Option<u64>,
    pub show_unattributed_warning: bool,
}

pub const UNATTRIBUTED_WARN_MIN_BYTES: u64 = 100 * 1024 * 1024;
pub const UNATTRIBUTED_WARN_MIN_RATIO: f64 = 0.10;

pub const POSSIBLE_CAUSES: &[&str] = &[
    "Linux page cache",
    "Filesystem cache",
    "VM memory",
    "Background processes",
];

/// Build a memory report from live system probes.
pub fn collect_memory_report() -> anyhow::Result<MemoryReport> {
    Ok(collect_memory_report_from(
        system_memory(),
        orbstack_vm_bytes(),
        if docker_available() {
            parse_container_stats()?
        } else {
            Vec::new()
        },
    ))
}

pub fn collect_memory_report_from(
    system: SystemMemorySnapshot,
    orbstack_vm_bytes: Option<u64>,
    containers: Vec<ContainerStat>,
) -> MemoryReport {
    let containers: Vec<ContainerMemory> = containers
        .into_iter()
        .map(|c| ContainerMemory {
            name: c.name,
            memory_bytes: c.memory_bytes,
            status: c.status,
        })
        .collect();
    let container_total_bytes = containers.iter().map(|c| c.memory_bytes).sum();
    let (unattributed_bytes, show_unattributed_warning) =
        compute_unattributed(orbstack_vm_bytes, container_total_bytes);
    MemoryReport {
        system,
        orbstack_vm_bytes,
        containers,
        container_total_bytes,
        unattributed_bytes,
        show_unattributed_warning,
    }
}

fn compute_unattributed(vm_bytes: Option<u64>, container_total_bytes: u64) -> (Option<u64>, bool) {
    let Some(vm) = vm_bytes else {
        return (None, false);
    };
    let gap = vm.saturating_sub(container_total_bytes);
    if gap == 0 {
        return (Some(0), false);
    }
    let ratio = gap as f64 / vm as f64;
    let warn = gap >= UNATTRIBUTED_WARN_MIN_BYTES && ratio >= UNATTRIBUTED_WARN_MIN_RATIO;
    (Some(gap), warn)
}

pub fn sort_containers(containers: &mut [ContainerMemory], sort: MemorySort) {
    containers.sort_by(|a, b| match sort {
        MemorySort::Memory => b
            .memory_bytes
            .cmp(&a.memory_bytes)
            .then_with(|| a.name.cmp(&b.name)),
        MemorySort::Name => a.name.cmp(&b.name),
        MemorySort::Status => a
            .status
            .cmp(&b.status)
            .then_with(|| b.memory_bytes.cmp(&a.memory_bytes)),
    });
}

pub fn container_total(containers: &[ContainerMemory]) -> u64 {
    containers.iter().map(|c| c.memory_bytes).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(total: u64, used: u64, avail: u64) -> SystemMemorySnapshot {
        SystemMemorySnapshot {
            total_bytes: total,
            used_bytes: used,
            available_bytes: avail,
        }
    }

    #[test]
    fn computes_unattributed_gap_and_warning() {
        let report = collect_memory_report_from(
            snap(128, 42, 86),
            Some(18_400_000_000),
            vec![
                ContainerStat {
                    name: "postgres".into(),
                    memory_bytes: 1_200_000_000,
                    status: "running".into(),
                },
                ContainerStat {
                    name: "redis".into(),
                    memory_bytes: 420_000_000,
                    status: "running".into(),
                },
                ContainerStat {
                    name: "api".into(),
                    memory_bytes: 850_000_000,
                    status: "running".into(),
                },
            ],
        );
        assert_eq!(report.container_total_bytes, 2_470_000_000);
        assert!(report.unattributed_bytes.unwrap() > 15_000_000_000);
        assert!(report.show_unattributed_warning);
    }

    #[test]
    fn no_warning_when_gap_small() {
        let report = collect_memory_report_from(
            snap(16, 8, 8),
            Some(1_000_000_000),
            vec![ContainerStat {
                name: "api".into(),
                memory_bytes: 950_000_000,
                status: "running".into(),
            }],
        );
        assert!(!report.show_unattributed_warning);
    }

    #[test]
    fn sort_by_memory_descending() {
        let mut containers = vec![
            ContainerMemory {
                name: "redis".into(),
                memory_bytes: 420,
                status: "running".into(),
            },
            ContainerMemory {
                name: "postgres".into(),
                memory_bytes: 1200,
                status: "running".into(),
            },
        ];
        sort_containers(&mut containers, MemorySort::Memory);
        assert_eq!(containers[0].name, "postgres");
    }

    #[test]
    fn sort_by_name() {
        let mut containers = vec![
            ContainerMemory {
                name: "redis".into(),
                memory_bytes: 420,
                status: "running".into(),
            },
            ContainerMemory {
                name: "api".into(),
                memory_bytes: 850,
                status: "running".into(),
            },
        ];
        sort_containers(&mut containers, MemorySort::Name);
        assert_eq!(containers[0].name, "api");
    }
}
