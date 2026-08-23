use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use directories::ProjectDirs;

/// Built-in macOS system processes that must never be killed.
#[cfg(target_os = "macos")]
const PROTECTED_BUILTIN: &[&str] = &[
    "kernel_task",
    "launchd",
    "WindowServer",
    "loginwindow",
    "SystemUIServer",
    "Finder",
    "Dock",
    // Core audio / indexing / security daemons
    "coreaudiod",
    "mds",
    "mds_stores",
    "mdworker",
    "mdworker_shared",
    "securityd",
    "bluetoothd",
    "powerd",
    "distnoted",
    "cfprefsd",
    "syslogd",
    "configd",
    "opendirectoryd",
    "UserEventAgent",
    "trustd",
    "hidd",
    "airportd",
];

/// Built-in Linux system processes that must never be killed.
#[cfg(target_os = "linux")]
const PROTECTED_BUILTIN: &[&str] = &[
    "systemd",
    "init",
    "sshd",
    "dbus-daemon",
    "NetworkManager",
    "cron",
    "crond",
    "rsyslogd",
    "polkitd",
    "agetty",
    "systemd-journald",
    "systemd-logind",
    "systemd-udevd",
    "systemd-resolved",
    "systemd-networkd",
    "kthreadd",
    "irqbalance",
    "udisksd",
    "cupsd",
];

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
const PROTECTED_BUILTIN: &[&str] = &[];

static USER_PROTECTED: OnceLock<HashSet<String>> = OnceLock::new();

pub fn protect_config_path() -> PathBuf {
    let dirs = ProjectDirs::from("com", "sweeper", "sweeper").expect("home directory");
    dirs.config_dir().join("protect.toml")
}

fn load_user_protected() -> HashSet<String> {
    let path = protect_config_path();
    parse_protect_file(&path).unwrap_or_default()
}

fn user_protected_names() -> &'static HashSet<String> {
    USER_PROTECTED.get_or_init(load_user_protected)
}

/// Parse `protect.toml`: one process name per line; `#` comments and blank lines ignored.
pub fn parse_protect_file(path: &Path) -> std::io::Result<HashSet<String>> {
    let text = fs::read_to_string(path)?;
    Ok(parse_protect_text(&text))
}

pub fn parse_protect_text(text: &str) -> HashSet<String> {
    text.lines()
        .map(|line| line.split('#').next().unwrap_or("").trim())
        .filter(|line| !line.is_empty())
        .map(|name| name.to_string())
        .collect()
}

pub fn is_protected(name: &str) -> bool {
    let base = basename(name);
    if PROTECTED_BUILTIN
        .iter()
        .any(|p| base.eq_ignore_ascii_case(p))
    {
        return true;
    }
    user_protected_names()
        .iter()
        .any(|p| base.eq_ignore_ascii_case(p))
}

fn basename(name: &str) -> &str {
    name.rsplit('/').next().unwrap_or(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn parse_protect_text_skips_comments_and_blanks() {
        let names = parse_protect_text("# my tools\nnode\n\n# another\nvite\n");
        assert_eq!(names.len(), 2);
        assert!(names.contains("node"));
        assert!(names.contains("vite"));
    }

    #[test]
    fn parse_protect_file_reads_lines() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "postgres\n# skip\nredis").unwrap();
        let names = parse_protect_file(file.path()).unwrap();
        assert!(names.contains("postgres"));
        assert!(names.contains("redis"));
        assert!(!names.contains("skip"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn builtin_protected_still_works() {
        assert!(is_protected("launchd"));
        assert!(is_protected("coreaudiod"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn builtin_protected_still_works() {
        assert!(is_protected("systemd"));
        assert!(is_protected("sshd"));
        assert!(is_protected("dbus-daemon"));
    }
}
