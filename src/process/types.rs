#[derive(Debug, Clone, PartialEq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
    pub cpu: f32,
    pub memory_bytes: u64,
    pub ports: Vec<u16>,
    pub command: Option<String>,
    pub cwd: Option<String>,
    /// Process uptime in seconds (from sysinfo snapshot).
    pub run_time_secs: u64,
    /// True when the kernel reports the process as zombie.
    pub is_zombie: bool,
}

impl ProcessInfo {
    pub fn memory_mb(&self) -> f64 {
        self.memory_bytes as f64 / (1024.0 * 1024.0)
    }
}
