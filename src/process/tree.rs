use std::collections::{HashMap, HashSet};

use crate::process::ProcessInfo;

/// Collect `roots` plus all descendants (via PPID), children before parents.
pub fn collect_tree_pids(procs: &[ProcessInfo], roots: &[u32]) -> Vec<u32> {
    let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
    for p in procs {
        children.entry(p.ppid).or_default().push(p.pid);
    }
    let mut seen = HashSet::new();
    let mut order = Vec::new();
    for &root in roots {
        dfs(root, &children, &mut seen, &mut order);
    }
    order
}

fn dfs(pid: u32, children: &HashMap<u32, Vec<u32>>, seen: &mut HashSet<u32>, out: &mut Vec<u32>) {
    if !seen.insert(pid) {
        return;
    }
    if let Some(kids) = children.get(&pid) {
        for &child in kids {
            dfs(child, children, seen, out);
        }
    }
    out.push(pid);
}

pub fn processes_for_pids<'a>(procs: &'a [ProcessInfo], pids: &[u32]) -> Vec<&'a ProcessInfo> {
    pids.iter()
        .filter_map(|pid| procs.iter().find(|p| p.pid == *pid))
        .collect()
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
    fn collects_descendants_children_first() {
        let procs = vec![
            proc(1, 0, "init"),
            proc(10, 1, "node"),
            proc(11, 10, "vite"),
            proc(12, 10, "esbuild"),
            proc(13, 11, "worker"),
            proc(20, 1, "other"),
        ];
        let order = collect_tree_pids(&procs, &[10]);
        assert_eq!(order, vec![13, 11, 12, 10]);
    }

    #[test]
    fn multiple_roots_dedupes_shared_nodes() {
        let procs = vec![proc(10, 1, "a"), proc(11, 10, "b"), proc(12, 11, "c")];
        // Selecting both 10 and 11 should not duplicate 11/12
        let order = collect_tree_pids(&procs, &[10, 11]);
        assert_eq!(order.iter().filter(|&&p| p == 12).count(), 1);
        assert_eq!(order.iter().filter(|&&p| p == 11).count(), 1);
        assert_eq!(order.last(), Some(&10));
    }

    #[test]
    fn unknown_root_still_included() {
        let procs = vec![proc(10, 1, "a")];
        let order = collect_tree_pids(&procs, &[99]);
        assert_eq!(order, vec![99]);
    }
}
