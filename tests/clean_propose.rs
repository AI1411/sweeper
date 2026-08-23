use sweeper::clean::{
    apply_excludes, propose_leftovers, CleanCandidate, IDLE_LISTENER_SECS, STALE_SERVER_SECS,
};
use sweeper::ProcessInfo;

fn proc(
    pid: u32,
    ppid: u32,
    name: &str,
    cpu: f32,
    run_time_secs: u64,
    ports: Vec<u16>,
    command: Option<&str>,
) -> ProcessInfo {
    ProcessInfo {
        pid,
        ppid,
        name: name.into(),
        cpu,
        memory_bytes: 0,
        ports,
        command: command.map(str::to_string),
        cwd: None,
        run_time_secs,
        is_zombie: false,
    }
}

fn zombie(pid: u32, ppid: u32, name: &str, run_time_secs: u64) -> ProcessInfo {
    ProcessInfo {
        pid,
        ppid,
        name: name.into(),
        cpu: 0.0,
        memory_bytes: 0,
        ports: vec![],
        command: None,
        cwd: None,
        run_time_secs,
        is_zombie: true,
    }
}

#[test]
fn proposes_stale_dev_server_on_listen_port() {
    let procs = vec![proc(
        100,
        50,
        "node",
        0.0,
        STALE_SERVER_SECS,
        vec![3000],
        None,
    )];
    let listening = vec![(3000, 100)];
    let out = propose_leftovers(&procs, &listening);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].process.pid, 100);
    assert!(out[0].reasons.iter().any(|r| r == "listening"));
    assert!(out[0].reasons.iter().any(|r| r == "stale-server"));
    assert!(out[0].reasons.iter().any(|r| r == "name:node"));
}

#[test]
fn proposes_orphan_dev_process() {
    let procs = vec![proc(200, 1, "vite", 0.0, 60, vec![], None)];
    let out = propose_leftovers(&procs, &[]);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].process.name, "vite");
    assert!(out[0].reasons.iter().any(|r| r == "orphan-ppid"));
}

#[test]
fn skips_dev_process_without_orphan_or_listen() {
    let procs = vec![
        proc(42, 1, "bash", 0.0, 3600, vec![], None),
        proc(300, 42, "node", 0.0, 60, vec![], None),
    ];
    let out = propose_leftovers(&procs, &[]);
    assert!(out.is_empty());
}

#[test]
fn skips_active_dev_listener_with_healthy_parent() {
    let procs = vec![
        proc(42, 1, "bash", 0.0, 3600, vec![], None),
        proc(400, 42, "node", 5.0, 120, vec![3000], None),
    ];
    let listening = vec![(3000, 400)];
    let out = propose_leftovers(&procs, &listening);
    assert!(out.is_empty());
}

#[test]
fn skips_non_dev_even_if_listening() {
    let procs = vec![proc(
        500,
        1,
        "nginx",
        0.0,
        STALE_SERVER_SECS,
        vec![80],
        None,
    )];
    let listening = vec![(80, 500)];
    let out = propose_leftovers(&procs, &listening);
    assert!(out.is_empty());
}

#[test]
fn matches_dev_name_case_insensitively() {
    let procs = vec![proc(600, 1, "Node", 0.0, 60, vec![], None)];
    let out = propose_leftovers(&procs, &[]);
    assert_eq!(out.len(), 1);
}

#[test]
fn detects_vite_from_command_line() {
    let procs = vec![proc(
        700,
        1,
        "node",
        0.0,
        60,
        vec![5173],
        Some("/usr/bin/node ./node_modules/.bin/vite"),
    )];
    let listening = vec![(5173, 700)];
    let out = propose_leftovers(&procs, &listening);
    assert_eq!(out.len(), 1);
    assert!(out[0].reasons.iter().any(|r| r == "stack:vite"));
}

#[test]
fn proposes_idle_listener() {
    let procs = vec![proc(
        800,
        50,
        "node",
        0.0,
        IDLE_LISTENER_SECS,
        vec![3000],
        None,
    )];
    let listening = vec![(3000, 800)];
    let out = propose_leftovers(&procs, &listening);
    assert_eq!(out.len(), 1);
    assert!(out[0].reasons.iter().any(|r| r == "idle-listener"));
}

#[test]
fn exclude_filters_by_name() {
    let cands = vec![
        CleanCandidate {
            process: proc(1, 1, "node", 0.0, 60, vec![], None),
            reasons: vec!["orphan-ppid".into()],
        },
        CleanCandidate {
            process: proc(2, 1, "python3", 0.0, 60, vec![], None),
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
        process: proc(1513, 1, "node", 0.0, 60, vec![3000], None),
        reasons: vec!["listening".into()],
    }];
    let out = apply_excludes(cands, &["1513".into()]);
    assert!(out.is_empty());
}

#[test]
fn exclude_filters_by_command() {
    let cands = vec![CleanCandidate {
        process: proc(
            1,
            1,
            "node",
            0.0,
            60,
            vec![],
            Some("node ./node_modules/.bin/vite"),
        ),
        reasons: vec!["orphan-ppid".into()],
    }];
    let out = apply_excludes(cands, &["vite".into()]);
    assert!(out.is_empty());
}

#[test]
fn detects_uvicorn_from_command_line() {
    let procs = vec![proc(
        900,
        1,
        "python",
        0.0,
        60,
        vec![8000],
        Some("uvicorn main:app --reload --port 8000"),
    )];
    let listening = vec![(8000, 900)];
    let out = propose_leftovers(&procs, &listening);
    assert_eq!(out.len(), 1);
    assert!(out[0].reasons.iter().any(|r| r == "stack:uvicorn"));
}

#[test]
fn detects_fastapi_from_command_line() {
    let procs = vec![proc(
        901,
        1,
        "python",
        0.0,
        60,
        vec![8000],
        Some("fastapi run app/main.py"),
    )];
    let listening = vec![(8000, 901)];
    let out = propose_leftovers(&procs, &listening);
    assert_eq!(out.len(), 1);
    assert!(out[0].reasons.iter().any(|r| r == "stack:fastapi"));
}

#[test]
fn detects_pnpm_from_command_line() {
    let procs = vec![proc(902, 1, "node", 0.0, 60, vec![], Some("pnpm dev"))];
    let out = propose_leftovers(&procs, &[]);
    assert_eq!(out.len(), 1);
    assert!(out[0].reasons.iter().any(|r| r == "stack:pnpm"));
    assert!(out[0].reasons.iter().any(|r| r == "orphan-ppid"));
}

#[test]
fn detects_astro_from_command_line() {
    let procs = vec![proc(
        903,
        1,
        "node",
        0.0,
        60,
        vec![4321],
        Some("node_modules/.bin/astro dev"),
    )];
    let listening = vec![(4321, 903)];
    let out = propose_leftovers(&procs, &listening);
    assert_eq!(out.len(), 1);
    assert!(out[0].reasons.iter().any(|r| r == "stack:astro"));
}

#[test]
fn detects_eslint_server_from_command_line() {
    let procs = vec![proc(
        904,
        1,
        "node",
        0.0,
        60,
        vec![],
        Some("node eslintServer.js --stdio"),
    )];
    let out = propose_leftovers(&procs, &[]);
    assert_eq!(out.len(), 1);
    assert!(out[0].reasons.iter().any(|r| r == "stack:eslint"));
}

#[test]
fn skips_young_active_launchd_orphan() {
    let procs = vec![proc(410, 1, "node", 5.0, 120, vec![3000], None)];
    let listening = vec![(3000, 410)];
    let out = propose_leftovers(&procs, &listening);
    assert!(out.is_empty());
}

#[test]
fn still_proposes_stale_launchd_orphan() {
    let procs = vec![proc(
        420,
        1,
        "node",
        0.0,
        STALE_SERVER_SECS,
        vec![3000],
        None,
    )];
    let listening = vec![(3000, 420)];
    let out = propose_leftovers(&procs, &listening);
    assert_eq!(out.len(), 1);
    assert!(out[0].reasons.iter().any(|r| r == "stale-server"));
}

#[test]
fn proposes_orphan_when_parent_shell_is_zombie() {
    let procs = vec![
        zombie(50, 1, "bash", 3600),
        proc(430, 50, "node", 0.0, 120, vec![3000], None),
    ];
    let listening = vec![(3000, 430)];
    let out = propose_leftovers(&procs, &listening);
    assert_eq!(out.len(), 1);
    assert!(out[0].reasons.iter().any(|r| r == "orphan-parent-defunct"));
}

#[test]
fn proposes_nested_worker_under_orphan_node() {
    let procs = vec![
        proc(500, 9999, "node", 0.0, 60, vec![3000], None),
        proc(501, 500, "vite", 0.0, 60, vec![], None),
    ];
    let listening = vec![(3000, 500)];
    let out = propose_leftovers(&procs, &listening);
    assert_eq!(out.len(), 2);
    assert!(out.iter().any(|c| c.process.pid == 500));
    assert!(out.iter().any(|c| c.process.pid == 501));
    let worker = out.iter().find(|c| c.process.pid == 501).unwrap();
    assert!(worker.reasons.iter().any(|r| r == "orphan-parent"));
}

#[test]
fn skips_postgres_and_redis_without_stale_signals() {
    let procs = vec![
        proc(905, 1, "postgres", 2.0, 3600, vec![5432], None),
        proc(906, 1, "redis-server", 1.0, 3600, vec![6379], None),
    ];
    let listening = vec![(5432, 905), (6379, 906)];
    let out = propose_leftovers(&procs, &listening);
    assert!(out.is_empty());
}
