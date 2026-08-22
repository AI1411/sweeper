use sweeper::ProcessInfo;

#[test]
fn memory_mb_converts_bytes() {
    let p = ProcessInfo {
        pid: 1,
        ppid: 0,
        name: "x".into(),
        cpu: 0.0,
        memory_bytes: 5 * 1024 * 1024,
        ports: vec![],
        command: None,
        cwd: None,
        run_time_secs: 0,
        is_zombie: false,
    };
    assert!((p.memory_mb() - 5.0).abs() < f64::EPSILON);
}
