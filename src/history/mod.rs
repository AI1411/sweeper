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
}

impl HistoryEntry {
    pub fn new(
        pid: u32,
        name: impl Into<String>,
        ports: Vec<u16>,
        signal: KillSignal,
        result: impl Into<String>,
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

pub fn load_entries() -> Result<Vec<HistoryEntry>> {
    load_entries_at(&history_path()?)
}

pub fn last_entry() -> Result<Option<HistoryEntry>> {
    Ok(load_entries()?.into_iter().next_back())
}
