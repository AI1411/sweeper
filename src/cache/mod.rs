use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use directories::{ProjectDirs, UserDirs};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheEntry {
    pub name: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub approximate: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheScanResult {
    pub entries: Vec<CacheEntry>,
    pub scan_duration_ms: u64,
}

const SCAN_CACHE_TTL: Duration = Duration::from_secs(3600);

#[derive(Debug, Serialize, Deserialize)]
struct CachedScan {
    scanned_at_ms: u128,
    entries: Vec<CachedEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedEntry {
    name: String,
    path: String,
    size_bytes: u64,
    approximate: bool,
}

pub fn collect_caches() -> Vec<CacheEntry> {
    collect_caches_scanned().entries
}

pub fn collect_caches_scanned() -> CacheScanResult {
    if let Some(cached) = load_cached_scan() {
        return cached;
    }

    eprintln!("Scanning dev caches…");
    let start = Instant::now();
    let mut entries: Vec<CacheEntry> = cache_providers().into_iter().flatten().collect();
    entries.sort_by_key(|e| std::cmp::Reverse(e.size_bytes));
    let result = CacheScanResult {
        scan_duration_ms: start.elapsed().as_millis() as u64,
        entries,
    };
    store_cached_scan(&result);
    result
}

fn cache_providers() -> Vec<Option<CacheEntry>> {
    vec![npm_cache(), cargo_cache(), pnpm_cache()]
}

fn home_dir() -> Option<PathBuf> {
    UserDirs::new().map(|d| d.home_dir().to_path_buf())
}

fn scan_cache_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "sweeper", "sweeper").map(|d| d.cache_dir().join("cache-scan.json"))
}

fn load_cached_scan() -> Option<CacheScanResult> {
    let path = scan_cache_path()?;
    let data = fs::read_to_string(&path).ok()?;
    let cached: CachedScan = serde_json::from_str(&data).ok()?;
    let age_ms = now_ms().saturating_sub(cached.scanned_at_ms);
    if age_ms > SCAN_CACHE_TTL.as_millis() {
        return None;
    }
    Some(CacheScanResult {
        scan_duration_ms: 0,
        entries: cached
            .entries
            .into_iter()
            .map(|e| CacheEntry {
                name: e.name,
                path: PathBuf::from(e.path),
                size_bytes: e.size_bytes,
                approximate: e.approximate,
            })
            .collect(),
    })
}

fn store_cached_scan(result: &CacheScanResult) {
    let Some(path) = scan_cache_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let cached = CachedScan {
        scanned_at_ms: now_ms(),
        entries: result
            .entries
            .iter()
            .map(|e| CachedEntry {
                name: e.name.clone(),
                path: e.path.display().to_string(),
                size_bytes: e.size_bytes,
                approximate: e.approximate,
            })
            .collect(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&cached) {
        let _ = fs::write(path, json);
    }
}

fn now_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn dir_size_du(path: &Path) -> Option<u64> {
    let path_str = path.to_str()?;
    let output = Command::new("du").args(["-sk", path_str]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let kb: u64 = text.split_whitespace().next()?.parse().ok()?;
    Some(kb.saturating_mul(1024))
}

pub fn dir_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    if path.is_file() {
        return fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    }
    if let Some(size) = dir_size_du(path) {
        return size;
    }
    dir_size_walk(path)
}

fn dir_size_walk(path: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let child = entry.path();
            if child.is_dir() {
                total += dir_size_walk(&child);
            } else {
                total += entry.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
    }
    total
}

fn cache_entry(name: &str, path: PathBuf) -> CacheEntry {
    let approximate = path.is_dir();
    let size_bytes = dir_size(&path);
    CacheEntry {
        name: name.into(),
        path,
        size_bytes,
        approximate,
    }
}

fn npm_cache() -> Option<CacheEntry> {
    let home = home_dir()?;
    let path = home.join(".npm");
    if !path.exists() {
        return None;
    }
    Some(cache_entry("npm", path))
}

fn cargo_cache() -> Option<CacheEntry> {
    let home = home_dir()?;
    let path = home.join(".cargo/registry");
    if !path.exists() {
        return None;
    }
    Some(cache_entry("cargo", path))
}

fn pnpm_cache() -> Option<CacheEntry> {
    let home = home_dir()?;
    let path = if cfg!(target_os = "macos") {
        home.join("Library/Caches/pnpm")
    } else {
        home.join(".local/share/pnpm/store")
    };
    if !path.exists() {
        return None;
    }
    Some(cache_entry("pnpm", path))
}

pub fn format_cache_report(entries: &[CacheEntry]) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    writeln!(out, "{}", crate::style::header("Dev Caches")).unwrap();
    writeln!(
        out,
        "{}",
        crate::style::dim("────────────────────────────────")
    )
    .unwrap();
    if entries.is_empty() {
        writeln!(out, "{}", crate::style::dim("No known dev caches found.")).unwrap();
        return out;
    }
    for entry in entries {
        let size = crate::memory::format_bytes(entry.size_bytes);
        let prefix = if entry.approximate { "~" } else { "" };
        writeln!(
            out,
            "{:<20} {}{}   {}",
            entry.name,
            prefix,
            size,
            entry.path.display()
        )
        .unwrap();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn dir_size_counts_nested_files() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::write(root.join("a.txt"), vec![0u8; 100]).unwrap();
        let sub = root.join("sub");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("b.txt"), vec![0u8; 250]).unwrap();
        assert_eq!(dir_size_walk(root), 350);
    }

    #[test]
    fn collect_from_temp_npm_layout() {
        let dir = tempfile::tempdir().unwrap();
        let npm = dir.path().join(".npm");
        fs::create_dir_all(&npm).unwrap();
        let mut file = fs::File::create(npm.join("cache.bin")).unwrap();
        file.write_all(&vec![0u8; 1024]).unwrap();

        let entry = CacheEntry {
            name: "npm".into(),
            path: npm.clone(),
            size_bytes: dir_size_walk(&npm),
            approximate: true,
        };
        assert_eq!(entry.size_bytes, 1024);
        let text = format_cache_report(&[entry]);
        assert!(text.contains("npm"));
        assert!(text.contains("Dev Caches"));
    }

    #[test]
    fn scan_result_records_duration() {
        let result = CacheScanResult {
            entries: vec![],
            scan_duration_ms: 42,
        };
        assert_eq!(result.scan_duration_ms, 42);
    }
}
