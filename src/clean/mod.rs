use std::collections::HashSet;

use crate::process::ProcessInfo;

const DEV_NAMES: &[&str] = &["node", "bun", "vite", "next-server", "python", "java"];

#[derive(Debug, Clone, PartialEq)]
pub struct CleanCandidate {
    pub process: ProcessInfo,
    pub reasons: Vec<String>,
}

pub fn propose_leftovers(procs: &[ProcessInfo], listening: &[(u16, u32)]) -> Vec<CleanCandidate> {
    let listen_pids: HashSet<u32> = listening.iter().map(|(_, pid)| *pid).collect();

    procs
        .iter()
        .filter_map(|p| {
            let name = p.name.to_lowercase();
            let mut reasons = Vec::new();
            let mut matched_dev = None;
            for d in DEV_NAMES {
                if name.contains(d) {
                    matched_dev = Some(*d);
                    break;
                }
            }
            let Some(dev) = matched_dev else {
                return None;
            };
            reasons.push(format!("name:{dev}"));
            let orphan = p.ppid == 1 || p.ppid == 0;
            let has_listen = listen_pids.contains(&p.pid);
            if orphan {
                reasons.push("orphan-ppid".into());
            }
            if has_listen {
                reasons.push("listening".into());
            }
            if !(orphan || has_listen) {
                return None;
            }
            Some(CleanCandidate {
                process: p.clone(),
                reasons,
            })
        })
        .collect()
}

/// Drop candidates whose name or pid string contains any exclude pattern (case-insensitive).
pub fn apply_excludes(cands: Vec<CleanCandidate>, excludes: &[String]) -> Vec<CleanCandidate> {
    if excludes.is_empty() {
        return cands;
    }
    let pats: Vec<String> = excludes.iter().map(|e| e.to_lowercase()).collect();
    cands
        .into_iter()
        .filter(|c| {
            let name = c.process.name.to_lowercase();
            let pid = c.process.pid.to_string();
            !pats
                .iter()
                .any(|p| name.contains(p) || pid.contains(p.as_str()))
        })
        .collect()
}

pub fn excludes_from_env() -> Vec<String> {
    std::env::var("SWEEPER_CLEAN_EXCLUDE")
        .ok()
        .map(|v| {
            v.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}
