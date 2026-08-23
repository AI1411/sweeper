use std::fs;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use super::format::{format_bytes, format_estimate};
use super::MemoryReport;

const DEFAULT_RETENTION_SECS: i64 = 24 * 60 * 60;
const DEFAULT_LEAK_WINDOW_SECS: i64 = 30 * 60;
const DEFAULT_LEAK_GROWTH_BYTES: u64 = 1_000_000_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemorySnapshot {
    pub timestamp: String,
    pub containers: Vec<ContainerSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContainerSnapshot {
    pub name: String,
    pub memory_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeakCandidate {
    pub container: String,
    pub start_bytes: u64,
    pub current_bytes: u64,
    pub growth_bytes: u64,
    pub elapsed_secs: u64,
    pub start_label: String,
}

pub fn snapshots_enabled() -> bool {
    match std::env::var("SWEEPER_MEMORY_SNAPSHOTS") {
        Ok(v) => !matches!(v.as_str(), "0" | "false" | "no"),
        Err(_) => true,
    }
}

pub fn leak_window_secs() -> i64 {
    std::env::var("SWEEPER_MEMORY_LEAK_WINDOW_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_LEAK_WINDOW_SECS)
}

pub fn leak_growth_threshold_bytes() -> u64 {
    std::env::var("SWEEPER_MEMORY_LEAK_GROWTH_BYTES")
        .ok()
        .and_then(|v| v.parse().ok())
        .or_else(|| {
            std::env::var("SWEEPER_MEMORY_LEAK_GROWTH_GB")
                .ok()
                .and_then(|v| v.parse::<f64>().ok())
                .map(|gb| (gb * 1_000_000_000.0) as u64)
        })
        .unwrap_or(DEFAULT_LEAK_GROWTH_BYTES)
}

pub fn snapshots_path() -> anyhow::Result<PathBuf> {
    let dirs = ProjectDirs::from("com", "sweeper", "sweeper").expect("home directory");
    let dir = dirs.data_dir();
    fs::create_dir_all(dir)?;
    Ok(dir.join("memory_snapshots.json"))
}

pub fn load_snapshots_at(path: &Path) -> anyhow::Result<Vec<MemorySnapshot>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(path)?;
    if data.trim().is_empty() {
        return Ok(Vec::new());
    }
    Ok(serde_json::from_str(&data)?)
}

pub fn save_snapshots_at(path: &Path, snapshots: &[MemorySnapshot]) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(snapshots)?)?;
    Ok(())
}

pub fn snapshot_from_report(report: &MemoryReport) -> MemorySnapshot {
    let timestamp = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| OffsetDateTime::UNIX_EPOCH.to_string());
    MemorySnapshot {
        timestamp,
        containers: report
            .containers
            .iter()
            .map(|c| ContainerSnapshot {
                name: c.name.clone(),
                memory_bytes: c.memory_bytes,
            })
            .collect(),
    }
}

pub fn prune_snapshots(snapshots: &mut Vec<MemorySnapshot>, retention_secs: i64) {
    let now = reference_time(snapshots);
    let cutoff = now - time::Duration::seconds(retention_secs);
    snapshots.retain(|s| {
        OffsetDateTime::parse(&s.timestamp, &Rfc3339)
            .map(|t| t >= cutoff)
            .unwrap_or(false)
    });
}

fn reference_time(snapshots: &[MemorySnapshot]) -> OffsetDateTime {
    snapshots
        .iter()
        .filter_map(|s| parse_ts(&s.timestamp))
        .max()
        .unwrap_or_else(OffsetDateTime::now_utc)
}

pub fn record_snapshot(report: &MemoryReport) -> anyhow::Result<()> {
    if !snapshots_enabled() {
        return Ok(());
    }
    let path = snapshots_path()?;
    let mut snapshots = load_snapshots_at(&path)?;
    snapshots.push(snapshot_from_report(report));
    prune_snapshots(&mut snapshots, DEFAULT_RETENTION_SECS);
    save_snapshots_at(&path, &snapshots)?;
    Ok(())
}

fn parse_ts(ts: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(ts, &Rfc3339).ok()
}

fn format_elapsed_label(elapsed_secs: u64) -> String {
    if elapsed_secs >= 3600 {
        let hours = elapsed_secs / 3600;
        format!("{hours} hr ago")
    } else if elapsed_secs >= 60 {
        let mins = elapsed_secs / 60;
        format!("{mins} min ago")
    } else {
        format!("{elapsed_secs} sec ago")
    }
}

/// Detect containers whose memory grew monotonically over the observation window.
pub fn detect_leaks(
    snapshots: &[MemorySnapshot],
    window_secs: i64,
    growth_threshold_bytes: u64,
) -> Vec<LeakCandidate> {
    detect_leaks_at(
        snapshots,
        window_secs,
        growth_threshold_bytes,
        reference_time(snapshots),
    )
}

pub fn detect_leaks_at(
    snapshots: &[MemorySnapshot],
    window_secs: i64,
    growth_threshold_bytes: u64,
    now: OffsetDateTime,
) -> Vec<LeakCandidate> {
    if snapshots.len() < 2 {
        return Vec::new();
    }
    let window_start = now - time::Duration::seconds(window_secs);

    let mut container_names: Vec<String> = Vec::new();
    for snap in snapshots {
        for c in &snap.containers {
            if !container_names.contains(&c.name) {
                container_names.push(c.name.clone());
            }
        }
    }

    let mut leaks = Vec::new();
    for name in container_names {
        let series: Vec<(OffsetDateTime, u64)> = snapshots
            .iter()
            .filter_map(|s| {
                let ts = parse_ts(&s.timestamp)?;
                if ts < window_start {
                    return None;
                }
                let bytes = s
                    .containers
                    .iter()
                    .find(|c| c.name == name)
                    .map(|c| c.memory_bytes)?;
                Some((ts, bytes))
            })
            .collect();
        if series.len() < 2 {
            continue;
        }
        let monotonic = series.windows(2).all(|w| w[1].1 >= w[0].1);
        if !monotonic {
            continue;
        }
        let (start_ts, start_bytes) = series[0];
        let (end_ts, end_bytes) = series[series.len() - 1];
        let growth = end_bytes.saturating_sub(start_bytes);
        if growth < growth_threshold_bytes {
            continue;
        }
        let elapsed_secs = (end_ts - start_ts).whole_seconds().unsigned_abs();
        leaks.push(LeakCandidate {
            container: name,
            start_bytes,
            current_bytes: end_bytes,
            growth_bytes: growth,
            elapsed_secs,
            start_label: format_elapsed_label(elapsed_secs),
        });
    }
    leaks.sort_by_key(|a| std::cmp::Reverse(a.growth_bytes));
    leaks
}

pub fn load_leak_candidates() -> anyhow::Result<Vec<LeakCandidate>> {
    let path = snapshots_path()?;
    let snapshots = load_snapshots_at(&path)?;
    Ok(detect_leaks(
        &snapshots,
        leak_window_secs(),
        leak_growth_threshold_bytes(),
    ))
}

pub fn format_leak_candidates(leaks: &[LeakCandidate]) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    if leaks.is_empty() {
        return out;
    }
    writeln!(out, "{}", crate::style::warn("⚠ Possible Memory Leak")).unwrap();
    for leak in leaks {
        writeln!(out, "container {}", leak.container).unwrap();
        writeln!(
            out,
            "{:<16} {}",
            leak.start_label,
            format_bytes(leak.start_bytes)
        )
        .unwrap();
        writeln!(out, "{:<16} {}", "now", format_bytes(leak.current_bytes)).unwrap();
        writeln!(
            out,
            "{:<16} +{}",
            "Growth",
            format_estimate(leak.growth_bytes)
        )
        .unwrap();
        writeln!(out).unwrap();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ts(offset_secs: i64) -> String {
        (OffsetDateTime::UNIX_EPOCH + time::Duration::seconds(offset_secs))
            .format(&Rfc3339)
            .unwrap()
    }

    fn snap(offset_secs: i64, name: &str, bytes: u64) -> MemorySnapshot {
        MemorySnapshot {
            timestamp: ts(offset_secs),
            containers: vec![ContainerSnapshot {
                name: name.into(),
                memory_bytes: bytes,
            }],
        }
    }

    #[test]
    fn detects_monotonic_growth_over_window() {
        let snapshots = vec![
            snap(0, "api", 820_000_000),
            snap(900, "api", 1_500_000_000),
            snap(1800, "api", 3_800_000_000),
        ];
        let leaks = detect_leaks(&snapshots, 1800, 1_000_000_000);
        assert_eq!(leaks.len(), 1);
        assert_eq!(leaks[0].container, "api");
        assert_eq!(leaks[0].growth_bytes, 2_980_000_000);
    }

    #[test]
    fn ignores_non_monotonic_series() {
        let snapshots = vec![
            snap(0, "api", 820_000_000),
            snap(900, "api", 2_000_000_000),
            snap(1800, "api", 1_500_000_000),
        ];
        let leaks = detect_leaks(&snapshots, 1800, 1_000_000_000);
        assert!(leaks.is_empty());
    }

    #[test]
    fn ignores_small_growth() {
        let snapshots = vec![snap(0, "api", 820_000_000), snap(1800, "api", 900_000_000)];
        let leaks = detect_leaks(&snapshots, 1800, 1_000_000_000);
        assert!(leaks.is_empty());
    }

    #[test]
    fn prune_drops_old_entries() {
        let base = OffsetDateTime::now_utc();
        let mut snapshots = vec![
            MemorySnapshot {
                timestamp: (base - time::Duration::seconds(5000))
                    .format(&Rfc3339)
                    .unwrap(),
                containers: vec![ContainerSnapshot {
                    name: "api".into(),
                    memory_bytes: 100,
                }],
            },
            MemorySnapshot {
                timestamp: (base - time::Duration::seconds(100))
                    .format(&Rfc3339)
                    .unwrap(),
                containers: vec![ContainerSnapshot {
                    name: "api".into(),
                    memory_bytes: 200,
                }],
            },
        ];
        prune_snapshots(&mut snapshots, 1000);
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].containers[0].memory_bytes, 200);
    }
}
