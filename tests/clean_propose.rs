use sweeper::clean::{apply_excludes, propose_leftovers, CleanCandidate};
use sweeper::ProcessInfo;

fn proc(pid: u32, ppid: u32, name: &str) -> ProcessInfo {
    ProcessInfo {
        pid,
        ppid,
        name: name.into(),
        cpu: 0.0,
        memory_bytes: 0,
        ports: Vec::new(),
        command: None,
        cwd: None,
    }
}

#[test]
fn proposes_dev_process_with_listen_port() {
    let procs = vec![proc(100, 50, "node")];
    let listening = vec![(3000, 100)];
    let out = propose_leftovers(&procs, &listening);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].process.pid, 100);
    assert!(out[0].reasons.iter().any(|r| r == "listening"));
    assert!(out[0].reasons.iter().any(|r| r == "name:node"));
}

#[test]
fn proposes_orphan_dev_process() {
    let procs = vec![proc(200, 1, "vite")];
    let out = propose_leftovers(&procs, &[]);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].process.name, "vite");
    assert!(out[0].reasons.iter().any(|r| r == "orphan-ppid"));
}

#[test]
fn skips_dev_process_without_orphan_or_listen() {
    let procs = vec![proc(300, 42, "node")];
    let out = propose_leftovers(&procs, &[]);
    assert!(out.is_empty());
}

#[test]
fn skips_non_dev_even_if_listening() {
    let procs = vec![proc(400, 1, "nginx")];
    let listening = vec![(80, 400)];
    let out = propose_leftovers(&procs, &listening);
    assert!(out.is_empty());
}

#[test]
fn matches_dev_name_case_insensitively() {
    let procs = vec![proc(500, 1, "Node")];
    let out = propose_leftovers(&procs, &[]);
    assert_eq!(out.len(), 1);
}

#[test]
fn exclude_filters_by_name() {
    let cands = vec![
        CleanCandidate {
            process: proc(1, 1, "node"),
            reasons: vec!["orphan-ppid".into()],
        },
        CleanCandidate {
            process: proc(2, 1, "python3"),
            reasons: vec!["orphan-ppid".into()],
        },
    ];
    let out = apply_excludes(cands, &["python".into()]);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].process.name, "node");
}

#[test]
fn exclude_filters_by_pid() {
    let cands = vec![CleanCandidate {
        process: proc(1513, 1, "node"),
        reasons: vec!["listening".into()],
    }];
    let out = apply_excludes(cands, &["1513".into()]);
    assert!(out.is_empty());
}
