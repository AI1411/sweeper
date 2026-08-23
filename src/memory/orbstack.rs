const ORBSTACK_VM_PROCESS_NAMES: &[&str] = &["OrbStack", "Linux"];

fn orbstack_vm_bytes_from_procs(procs: &[crate::process::ProcessInfo]) -> Option<u64> {
    let mut best: u64 = 0;
    for p in procs {
        let name_l = p.name.to_lowercase();
        if ORBSTACK_VM_PROCESS_NAMES
            .iter()
            .any(|hint| name_l.contains(&hint.to_lowercase()))
        {
            best = best.max(p.memory_bytes);
        }
    }
    if best > 0 {
        Some(best)
    } else {
        None
    }
}

/// Estimate OrbStack VM memory from running processes (macOS only).
pub fn orbstack_vm_bytes() -> Option<u64> {
    if cfg!(target_os = "macos") {
        let procs = crate::process::list::list_processes();
        orbstack_vm_bytes_from_procs(&procs)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::ProcessInfo;

    #[test]
    fn picks_largest_orbstack_candidate() {
        let procs = vec![
            ProcessInfo {
                pid: 1,
                ppid: 0,
                name: "OrbStack".into(),
                cpu: 0.0,
                memory_bytes: 18_400_000_000,
                ports: vec![],
                command: None,
                cwd: None,
                run_time_secs: 0,
                is_zombie: false,
            },
            ProcessInfo {
                pid: 2,
                ppid: 1,
                name: "Linux".into(),
                cpu: 0.0,
                memory_bytes: 1_000_000,
                ports: vec![],
                command: None,
                cwd: None,
                run_time_secs: 0,
                is_zombie: false,
            },
        ];
        assert_eq!(orbstack_vm_bytes_from_procs(&procs), Some(18_400_000_000));
    }
}
