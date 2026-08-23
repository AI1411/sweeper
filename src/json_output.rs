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
