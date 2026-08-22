use sweeper::clean::propose_leftovers;
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
    assert_eq!(out[0].pid, 100);
}

#[test]
fn proposes_orphan_dev_process() {
    let procs = vec![proc(200, 1, "vite")];
    let out = propose_leftovers(&procs, &[]);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].name, "vite");
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
