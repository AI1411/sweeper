use std::fs;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::error::Result;

const MAX_ENTRIES: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum KillSignal {
    Term,
    Kill,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HistoryEntry {
    /// RFC3339 timestamp
    pub time: String,
    pub pid: u32,
    pub name: String,
    pub ports: Vec<u16>,
    pub signal: KillSignal,
    pub result: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

impl HistoryEntry {
    pub fn new(
        pid: u32,
        name: impl Into<String>,
        ports: Vec<u16>,
        signal: KillSignal,
        result: impl Into<String>,
    ) -> Self {
        Self::with_project(pid, name, ports, signal, result, None)
    }

    pub fn with_project(
        pid: u32,
        name: impl Into<String>,
        ports: Vec<u16>,
        signal: KillSignal,
        result: impl Into<String>,
        project: Option<String>,
    ) -> Self {
        let time = OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .unwrap_or_else(|_| "unknown".into());
        Self {
            time,
            pid,
            name: name.into(),
            ports,
            signal,
            result: result.into(),
            project,
        }
    }
}

pub fn history_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("com", "sweeper", "sweeper").expect("home directory");
    let dir = dirs.data_dir();
    fs::create_dir_all(dir)?;
    Ok(dir.join("history.json"))
}

pub fn load_entries_at(path: &Path) -> Result<Vec<HistoryEntry>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(path)?;
    if data.trim().is_empty() {
        return Ok(Vec::new());
    }
    Ok(serde_json::from_str(&data)?)
}

pub fn append_entry_at(path: &Path, entry: HistoryEntry) -> Result<()> {
    let mut entries = load_entries_at(path)?;
    entries.push(entry);
    if entries.len() > MAX_ENTRIES {
        let skip = entries.len() - MAX_ENTRIES;
        entries = entries.into_iter().skip(skip).collect();
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(&entries)?)?;
    Ok(())
}

pub fn append_entry(entry: HistoryEntry) -> Result<()> {
    append_entry_at(&history_path()?, entry)
}

/// Build a history entry with optional project name from process metadata.
pub fn entry_for_process(
    pid: u32,
    name: &str,
    ports: Vec<u16>,
    signal: KillSignal,
    result: impl Into<String>,
    proc: Option<&crate::process::ProcessInfo>,
) -> HistoryEntry {
    let project = proc.and_then(|p| crate::project::infer_project(p).map(|(n, _)| n));
    HistoryEntry::with_project(pid, name, ports, signal, result, project)
}

pub fn load_entries() -> Result<Vec<HistoryEntry>> {
    load_entries_at(&history_path()?)
}

pub fn last_entry() -> Result<Option<HistoryEntry>> {
    Ok(load_entries()?.into_iter().next_back())
}

/// Parse duration strings like `1h`, `30m`, `2d` into seconds.
pub fn parse_since_duration(input: &str) -> Option<u64> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }
    let (num, unit) = input.split_at(input.len().saturating_sub(1));
    let value: u64 = num.parse().ok()?;
    match unit {
        "s" => Some(value),
        "m" => Some(value * 60),
        "h" => Some(value * 3600),
        "d" => Some(value * 86400),
        _ => {
            let value: u64 = input.parse().ok()?;
            Some(value * 3600)
        }
    }
}

pub fn filter_entries(
    entries: Vec<HistoryEntry>,
    project: Option<&str>,
    since_secs: Option<u64>,
    limit: Option<usize>,
) -> Vec<HistoryEntry> {
    let now = OffsetDateTime::now_utc();
    let mut out: Vec<HistoryEntry> = entries
        .into_iter()
        .filter(|e| {
            if let Some(q) = project {
                let q = q.to_lowercase();
                let name_match = e.name.to_lowercase().contains(&q);
                let project_match = e
                    .project
                    .as_ref()
                    .map(|p| p.to_lowercase().contains(&q))
                    .unwrap_or(false);
                if !name_match && !project_match {
                    return false;
                }
            }
            if let Some(secs) = since_secs {
                if let Ok(ts) = OffsetDateTime::parse(&e.time, &Rfc3339) {
                    let age = (now - ts).whole_seconds().max(0) as u64;
                    if age > secs {
                        return false;
                    }
                }
            }
            true
        })
        .collect();
    if let Some(n) = limit {
        if out.len() > n {
            let skip = out.len() - n;
            out = out.into_iter().skip(skip).collect();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_since_duration_units() {
        assert_eq!(parse_since_duration("1h"), Some(3600));
        assert_eq!(parse_since_duration("30m"), Some(1800));
        assert_eq!(parse_since_duration("2d"), Some(172800));
    }

    #[test]
    fn filter_by_project_name() {
        let entries = vec![
            HistoryEntry::with_project(
                1,
                "node",
                vec![],
                KillSignal::Term,
                "ok",
                Some("my-app".into()),
            ),
            HistoryEntry::with_project(
                2,
                "vite",
                vec![],
                KillSignal::Term,
                "ok",
                Some("other".into()),
            ),
        ];
        let filtered = filter_entries(entries, Some("my-app"), None, None);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].pid, 1);
    }

    #[test]
    fn legacy_entries_deserialize_without_project() {
        let json = r#"[{"time":"2026-01-01T00:00:00Z","pid":1,"name":"node","ports":[],"signal":"term","result":"Terminated"}]"#;
        let entries: Vec<HistoryEntry> = serde_json::from_str(json).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].project.is_none());
    }
}
