use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::process::ProcessInfo;

/// Detect tmux/screen session label for a process from parent chain or environment.
pub fn infer_session_label(
    proc: &ProcessInfo,
    by_pid: &HashMap<u32, &ProcessInfo>,
) -> Option<String> {
    if let Some(label) = session_from_environ(proc.pid) {
        return Some(label);
    }
    session_from_parent_chain(proc, by_pid)
}

fn session_from_parent_chain(
    proc: &ProcessInfo,
    by_pid: &HashMap<u32, &ProcessInfo>,
) -> Option<String> {
    let mut current_ppid = proc.ppid;
    let mut seen = HashSet::new();
    while current_ppid != 0 && seen.insert(current_ppid) {
        let Some(parent) = by_pid.get(&current_ppid) else {
            break;
        };
        if let Some(label) = session_label_from_process_name(&parent.name) {
            return Some(label);
        }
        if let Some(label) = session_from_environ(parent.pid) {
            return Some(label);
        }
        current_ppid = parent.ppid;
    }
    None
}

fn session_label_from_process_name(name: &str) -> Option<String> {
    let n = name.to_lowercase();
    if n.contains("tmux") {
        return Some("tmux".into());
    }
    if n.contains("screen") {
        return Some("screen".into());
    }
    None
}

fn session_from_environ(pid: u32) -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        return linux_session_from_environ(pid);
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = pid;
        None
    }
}

#[cfg(target_os = "linux")]
fn linux_session_from_environ(pid: u32) -> Option<String> {
    let data = std::fs::read(format!("/proc/{pid}/environ")).ok()?;
    for entry in data.split(|b| *b == 0) {
        let Ok(text) = std::str::from_utf8(entry) else {
            continue;
        };
        if let Some(tmux) = text.strip_prefix("TMUX=") {
            return parse_tmux_env(tmux);
        }
    }
    None
}

fn parse_tmux_env(value: &str) -> Option<String> {
    // TMUX=/tmp/tmux-1000/default,12345,0
    let path = value.split(',').next()?;
    let session = Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())?;
    Some(format!("tmux:{session}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proc(pid: u32, ppid: u32, name: &str) -> ProcessInfo {
        ProcessInfo {
            pid,
            ppid,
            name: name.into(),
            cpu: 0.0,
            memory_bytes: 0,
            ports: vec![],
            command: None,
            cwd: None,
            run_time_secs: 0,
            is_zombie: false,
        }
    }

    #[test]
    fn detects_tmux_parent() {
        let tmux = proc(10, 1, "tmux");
        let child = proc(100, 10, "node");
        let by_pid: HashMap<u32, &ProcessInfo> = [(10, &tmux)].into_iter().collect();
        let label = infer_session_label(&child, &by_pid).unwrap();
        assert_eq!(label, "tmux");
    }

    #[test]
    fn parse_tmux_env_value() {
        assert_eq!(
            parse_tmux_env("/tmp/tmux-1000/api-dev,48291,0"),
            Some("tmux:api-dev".into())
        );
    }

    #[test]
    fn no_session_without_multiplexer() {
        let bash = proc(10, 1, "bash");
        let child = proc(100, 10, "node");
        let by_pid: HashMap<u32, &ProcessInfo> = [(10, &bash)].into_iter().collect();
        assert!(infer_session_label(&child, &by_pid).is_none());
    }
}
