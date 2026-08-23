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
pub struct CleanJson {
    pub candidates: Vec<CleanCandidateJson>,
    pub summary: CleanSummaryJson,
}

#[derive(Debug, Serialize)]
pub struct ProjectJson {
    pub name: String,
    pub path: String,
    pub process_count: usize,
    pub memory_bytes: u64,
    pub ports: Vec<u16>,
}

#[derive(Debug, Serialize)]
pub struct HistoryJson {
    pub time: String,
    pub pid: u32,
    pub name: String,
    pub ports: Vec<u16>,
    pub signal: String,
    pub result: String,
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
pub struct MemoryJson {
    pub system: SystemMemoryJson,
    pub orbstack_vm_bytes: Option<u64>,
    pub containers: Vec<ContainerMemoryJson>,
    pub container_total_bytes: u64,
    pub unattributed_bytes: Option<u64>,
    pub show_unattributed_warning: bool,
    pub possible_causes: Vec<&'static str>,
}

impl From<&crate::memory::MemoryReport> for MemoryJson {
    fn from(report: &crate::memory::MemoryReport) -> Self {
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
        }
    }
}
