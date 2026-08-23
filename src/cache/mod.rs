use std::fs;
use std::path::{Path, PathBuf};

use directories::UserDirs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheEntry {
    pub name: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub approximate: bool,
}

pub fn collect_caches() -> Vec<CacheEntry> {
    let mut entries: Vec<CacheEntry> = cache_providers()
        .into_iter()
        .flatten()
        .collect();
    entries.sort_by_key(|e| std::cmp::Reverse(e.size_bytes));
    entries
}

fn cache_providers() -> Vec<Option<CacheEntry>> {
    vec![npm_cache(), cargo_cache(), pnpm_cache()]
}

fn home_dir() -> Option<PathBuf> {
    UserDirs::new().map(|d| d.home_dir().to_path_buf())
}

pub fn dir_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    let mut total = 0u64;
    if path.is_file() {
        return fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    }
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let child = entry.path();
            if child.is_dir() {
                total += dir_size(&child);
            } else {
                total += entry.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
    }
    total
}

fn cache_entry(name: &str, path: PathBuf) -> CacheEntry {
    let size_bytes = dir_size(&path);
    CacheEntry {
        name: name.into(),
        path,
        size_bytes,
        approximate: true,
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
    writeln!(out, "{}", crate::style::dim("────────────────────────────────")).unwrap();
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
        assert_eq!(dir_size(root), 350);
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
            size_bytes: dir_size(&npm),
            approximate: true,
        };
        assert_eq!(entry.size_bytes, 1024);
        let text = format_cache_report(&[entry]);
        assert!(text.contains("npm"));
        assert!(text.contains("Dev Caches"));
    }
}
