use std::collections::{HashMap, HashSet};

use crate::process::ProcessInfo;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeRow {
    pub process_index: usize,
    pub depth: usize,
    pub prefix: String,
}

pub fn layout_tree_rows(procs: &[ProcessInfo], indices: &[usize]) -> Vec<TreeRow> {
    if indices.is_empty() {
        return Vec::new();
    }
    let mut visible_pids = HashSet::with_capacity(indices.len());
    for &idx in indices {
        visible_pids.insert(procs[idx].pid);
    }
    let mut children: HashMap<u32, Vec<usize>> = HashMap::new();
    for &idx in indices {
        children.entry(procs[idx].ppid).or_default().push(idx);
    }
    for kids in children.values_mut() {
        kids.sort_by_key(|&idx| (procs[idx].name.to_lowercase(), procs[idx].pid));
    }
    let mut roots: Vec<usize> = indices
        .iter()
        .copied()
        .filter(|&idx| !visible_pids.contains(&procs[idx].ppid))
        .collect();
    roots.sort_by_key(|&idx| (procs[idx].name.to_lowercase(), procs[idx].pid));
    let mut rows = Vec::with_capacity(indices.len());
    for (i, &root) in roots.iter().enumerate() {
        walk_tree(
            procs,
            &children,
            root,
            0,
            &[],
            i + 1 == roots.len(),
            &mut rows,
        );
    }
    rows
}

fn walk_tree(
    procs: &[ProcessInfo],
    children: &HashMap<u32, Vec<usize>>,
    idx: usize,
    depth: usize,
    ancestor_continues: &[bool],
    is_last: bool,
    out: &mut Vec<TreeRow>,
) {
    out.push(TreeRow {
        process_index: idx,
        depth,
        prefix: tree_prefix(depth, ancestor_continues, is_last),
    });
    let kids = children
        .get(&procs[idx].pid)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let mut continues = ancestor_continues.to_vec();
    if depth > 0 {
        continues.push(!is_last);
    }
    for (i, &child) in kids.iter().enumerate() {
        walk_tree(
            procs,
            children,
            child,
            depth + 1,
            &continues,
            i + 1 == kids.len(),
            out,
        );
    }
}

fn tree_prefix(depth: usize, ancestor_continues: &[bool], is_last: bool) -> String {
    if depth == 0 {
        return String::new();
    }
    let mut prefix = String::new();
    for &continues in ancestor_continues.iter().take(depth.saturating_sub(1)) {
        prefix.push_str(if continues { "│ " } else { "  " });
    }
    prefix.push_str(if is_last { "└─ " } else { "├─ " });
    prefix
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
        assert_eq!(collect_tree_pids(&procs, &[10]), vec![13, 11, 12, 10]);
    }
    #[test]
    fn layout_tree_rows_nests_children() {
        let procs = vec![
            proc(1, 0, "init"),
            proc(10, 1, "node"),
            proc(11, 10, "vite"),
            proc(12, 10, "esbuild"),
            proc(20, 1, "other"),
        ];
        let rows = layout_tree_rows(&procs, &(0..procs.len()).collect::<Vec<_>>());
        assert_eq!(rows.len(), 5);
        assert!(rows.iter().any(|r| r.depth == 2));
    }
    #[test]
    fn layout_tree_rows_empty() {
        assert!(layout_tree_rows(&[proc(1, 0, "a")], &[]).is_empty());
    }
}
