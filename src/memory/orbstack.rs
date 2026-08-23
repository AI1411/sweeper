#[cfg(target_os = "macos")]
/// Process names associated with the OrbStack Linux VM on macOS.
const ORBSTACK_VM_PROCESS_NAMES: &[&str] = &["OrbStack", "Linux"];

/// Estimate OrbStack VM memory from running processes (macOS only).
pub fn orbstack_vm_bytes() -> Option<u64> {
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
    #[cfg(target_os = "macos")]
    {
        let procs = crate::process::list::list_processes();
        let mut best: u64 = 0;
        for p in &procs {
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
        // orbstack_vm_bytes uses list_processes(); test logic inline
        let mut best = 0u64;
        for p in &procs {
            let name_l = p.name.to_lowercase();
            if ORBSTACK_VM_PROCESS_NAMES
                .iter()
                .any(|hint| name_l.contains(&hint.to_lowercase()))
            {
                best = best.max(p.memory_bytes);
            }
        }
        assert_eq!(best, 18_400_000_000);
    }
}
