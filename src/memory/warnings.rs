use super::ContainerMemory;

pub const DEFAULT_WARN_BYTES: u64 = 2 * 1024 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryWarning {
    pub container: String,
    pub memory_bytes: u64,
    pub kind: &'static str,
}

pub fn warn_threshold_bytes(cli_bytes: Option<u64>) -> u64 {
    if let Some(b) = cli_bytes {
        return b;
    }
    if let Ok(v) = std::env::var("SWEEPER_MEMORY_WARN_GB") {
        if let Ok(gb) = v.parse::<f64>() {
            return (gb * 1024.0 * 1024.0 * 1024.0) as u64;
        }
    }
    DEFAULT_WARN_BYTES
}

pub fn parse_warn_threshold(token: &str) -> Option<u64> {
    let t = token.trim().to_lowercase();
    if t.ends_with("gb") {
        let n: f64 = t.trim_end_matches("gb").trim().parse().ok()?;
        return Some((n * 1024.0 * 1024.0 * 1024.0) as u64);
    }
    if t.ends_with("mb") {
        let n: f64 = t.trim_end_matches("mb").trim().parse().ok()?;
        return Some((n * 1024.0 * 1024.0) as u64);
    }
    t.parse::<u64>().ok()
}

pub fn high_memory_warnings(
    containers: &[ContainerMemory],
    threshold_bytes: u64,
) -> Vec<MemoryWarning> {
    containers
        .iter()
        .filter(|c| c.memory_bytes >= threshold_bytes)
        .map(|c| MemoryWarning {
            container: c.name.clone(),
            memory_bytes: c.memory_bytes,
            kind: "high_usage",
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warns_above_threshold() {
        let containers = vec![
            ContainerMemory {
                name: "api".into(),
                memory_bytes: 5 * 1024 * 1024 * 1024,
                status: "running".into(),
            },
            ContainerMemory {
                name: "redis".into(),
                memory_bytes: 100 * 1024 * 1024,
                status: "running".into(),
            },
        ];
        let warnings = high_memory_warnings(&containers, DEFAULT_WARN_BYTES);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].container, "api");
    }

    #[test]
    fn parses_gb_threshold() {
        assert_eq!(parse_warn_threshold("2gb"), Some(2 * 1024 * 1024 * 1024));
    }
}
