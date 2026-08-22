use sweeper::process::ports::merge_ports;
use sweeper::ProcessInfo;

fn proc(pid: u32, name: &str) -> ProcessInfo {
    ProcessInfo {
        pid,
        ppid: 1,
        name: name.into(),
        cpu: 0.0,
        memory_bytes: 0,
        ports: Vec::new(),
        command: None,
        cwd: None,
        run_time_secs: 0,
        is_zombie: false,
    }
}

#[test]
fn merges_ports_onto_matching_pids() {
    let mut procs = vec![proc(10, "node"), proc(20, "python")];
    merge_ports(&mut procs, &[(3000, 10), (8000, 20), (3001, 10)]);
    assert_eq!(procs[0].ports, vec![3000, 3001]);
    assert_eq!(procs[1].ports, vec![8000]);
}

#[test]
fn does_not_duplicate_ports() {
    let mut procs = vec![proc(10, "node")];
    merge_ports(&mut procs, &[(3000, 10)]);
    merge_ports(&mut procs, &[(3000, 10)]);
    assert_eq!(procs[0].ports, vec![3000]);
}

#[test]
fn ignores_unknown_pids() {
    let mut procs = vec![proc(10, "node")];
    merge_ports(&mut procs, &[(9999, 999)]);
    assert!(procs[0].ports.is_empty());
}
