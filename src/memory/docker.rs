use std::collections::HashMap;
use std::process::Command;

use super::format::parse_docker_bytes;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerStat {
    pub name: String,
    pub memory_bytes: u64,
    pub status: String,
}

pub fn docker_available() -> bool {
    Command::new("docker")
        .args(["info", "--format", "{{.ServerVersion}}"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Parse `docker stats --no-stream` and `docker ps` fixture or live output.
pub fn parse_container_stats_from(
    stats_output: &str,
    ps_output: &str,
) -> anyhow::Result<Vec<ContainerStat>> {
    let statuses = parse_ps_status(ps_output);
    let mut out = Vec::new();
    for line in stats_output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let name = parts.next().unwrap_or_default().to_string();
        let mem_field = parts.next().unwrap_or_default();
        let usage = mem_field.split('/').next().unwrap_or(mem_field);
        let memory_bytes = parse_docker_bytes(usage)
            .ok_or_else(|| anyhow::anyhow!("invalid docker memory field: {mem_field}"))?;
        let status = statuses
            .get(&name)
            .cloned()
            .unwrap_or_else(|| "unknown".into());
        out.push(ContainerStat {
            name,
            memory_bytes,
            status,
        });
    }
    Ok(out)
}

fn parse_ps_status(ps_output: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in ps_output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((name, status)) = line.split_once('\t') {
            let short = status
                .split_whitespace()
                .next()
                .unwrap_or(status)
                .to_string();
            map.insert(name.to_string(), short);
        }
    }
    map
}

pub fn parse_container_stats() -> anyhow::Result<Vec<ContainerStat>> {
    let stats = Command::new("docker")
        .args([
            "stats",
            "--no-stream",
            "--format",
            "{{.Name}}\t{{.MemUsage}}",
        ])
        .output()?;
    if !stats.status.success() {
        anyhow::bail!(
            "docker stats failed: {}",
            String::from_utf8_lossy(&stats.stderr)
        );
    }
    let ps = Command::new("docker")
        .args(["ps", "-a", "--format", "{{.Names}}\t{{.Status}}"])
        .output()?;
    if !ps.status.success() {
        anyhow::bail!("docker ps failed: {}", String::from_utf8_lossy(&ps.stderr));
    }
    parse_container_stats_from(
        &String::from_utf8_lossy(&stats.stdout),
        &String::from_utf8_lossy(&ps.stdout),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const STATS_FIXTURE: &str = "\
postgres\t1.2GiB / 7.776GiB
redis\t420MiB / 7.776GiB
api\t850MiB / 7.776GiB
";

    const PS_FIXTURE: &str = "\
postgres\tUp 2 hours
redis\tUp 2 hours
api\tUp 2 hours
";

    #[test]
    fn parses_stats_and_status_fixture() {
        let stats = parse_container_stats_from(STATS_FIXTURE, PS_FIXTURE).unwrap();
        assert_eq!(stats.len(), 3);
        assert_eq!(stats[0].name, "postgres");
        assert_eq!(stats[0].status, "Up");
        assert!(stats[0].memory_bytes > 1_000_000_000);
    }

    #[test]
    fn ignores_limit_after_slash() {
        let stats = parse_container_stats_from("api\t850MiB / 7.776GiB\n", "").unwrap();
        assert_eq!(stats[0].memory_bytes, parse_docker_bytes("850MiB").unwrap());
    }
}
