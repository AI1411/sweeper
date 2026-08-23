use std::process::Command;

use crate::memory::parse_docker_bytes;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockerDiskRow {
    pub kind: String,
    pub total_bytes: u64,
    pub reclaimable_bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockerDiskReport {
    pub rows: Vec<DockerDiskRow>,
    pub total_bytes: u64,
    pub reclaimable_bytes: u64,
}

pub fn docker_disk_available() -> bool {
    Command::new("docker")
        .args(["info", "--format", "{{.ServerVersion}}"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn parse_docker_system_df(output: &str) -> anyhow::Result<DockerDiskReport> {
    let mut rows = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("TYPE") {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }
        let kind = parts[0].to_string();
        let total_bytes = parse_size_token(parts[3])?;
        let reclaimable_bytes = parts.get(4).and_then(|r| parse_reclaimable_token(r));
        rows.push(DockerDiskRow {
            kind,
            total_bytes,
            reclaimable_bytes,
        });
    }
    let total_bytes = rows.iter().map(|r| r.total_bytes).sum();
    let reclaimable_bytes = rows.iter().map(|r| r.reclaimable_bytes.unwrap_or(0)).sum();
    Ok(DockerDiskReport {
        rows,
        total_bytes,
        reclaimable_bytes,
    })
}

fn parse_size_token(token: &str) -> anyhow::Result<u64> {
    parse_docker_bytes(token).ok_or_else(|| anyhow::anyhow!("invalid size: {token}"))
}

fn parse_reclaimable_token(token: &str) -> Option<u64> {
    let value = token.split('(').next().unwrap_or(token).trim();
    parse_docker_bytes(value)
}

pub fn collect_docker_disk_report() -> anyhow::Result<DockerDiskReport> {
    let output = Command::new("docker").args(["system", "df"]).output()?;
    if !output.status.success() {
        anyhow::bail!(
            "docker system df failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    parse_docker_system_df(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = "\
TYPE            TOTAL     ACTIVE    SIZE      RECLAIMABLE
Images          10        5         8.4GB     3.1GB (37%)
Containers      5         2         1.2GB     800MB (66%)
Local Volumes   3         3         12.8GB    0B (0%)
Build Cache     0         0         21.4GB    18.2GB
";

    #[test]
    fn parses_docker_system_df_fixture() {
        let report = parse_docker_system_df(FIXTURE).unwrap();
        assert_eq!(report.rows.len(), 4);
        assert!(report.total_bytes > 0);
        assert!(report.reclaimable_bytes > 0);
        assert_eq!(report.rows[0].kind, "Images");
        assert!(report.rows[0].reclaimable_bytes.unwrap() > 0);
    }
}
