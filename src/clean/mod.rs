use crate::process::ProcessInfo;

const DEV_NAMES: &[&str] = &["node", "bun", "vite", "next-server", "python", "java"];

pub fn propose_leftovers(procs: &[ProcessInfo], listening: &[(u16, u32)]) -> Vec<ProcessInfo> {
    let listen_pids: std::collections::HashSet<u32> =
        listening.iter().map(|(_, pid)| *pid).collect();

    procs
        .iter()
        .filter(|p| {
            let name = p.name.to_lowercase();
            let is_dev = DEV_NAMES.iter().any(|d| name.contains(d));
            let orphan = p.ppid == 1 || p.ppid == 0;
            let has_listen = listen_pids.contains(&p.pid);
            is_dev && (orphan || has_listen)
        })
        .cloned()
        .collect()
}
