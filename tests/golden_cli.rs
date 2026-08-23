use sweeper::clean::{format_reasons_display, CleanCandidate, CleanSummary};
use sweeper::commands::clean::format_summary_lines;
use sweeper::commands::ports_list::format_ports_table;
use sweeper::process::kill::KillOutcome;
use sweeper::process::ProcessInfo;
use sweeper::report::{format_kill_summary, KillResult};

const GOLDEN_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");

fn setup_no_color() {
    std::env::set_var("NO_COLOR", "1");
}

fn golden_compare(name: &str, actual: &str) {
    let path = format!("{GOLDEN_DIR}/{name}.txt");
    if std::env::var("UPDATE_GOLDEN").ok().as_deref() == Some("1") {
        std::fs::write(&path, actual).expect("write golden");
        return;
    }
    let expected = std::fs::read_to_string(&path).expect("read golden");
    assert_eq!(actual, expected, "golden mismatch for {name}");
}

fn fixture_proc() -> ProcessInfo {
    ProcessInfo {
        pid: 48291,
        ppid: 4701,
        name: "node".into(),
        cpu: 0.2,
        memory_bytes: 128 * 1024 * 1024,
        ports: vec![3000],
        command: Some("./node_modules/.bin/vite".into()),
        cwd: Some("/Users/dev/my-app".into()),
        run_time_secs: 8040,
        is_zombie: false,
    }
}

#[test]
fn golden_clean_reasons_display() {
    setup_no_color();
    let candidate = CleanCandidate {
        process: fixture_proc(),
        reasons: vec!["stale-server".into(), "orphan-parent".into()],
    };
    let lines = format_reasons_display(&candidate);
    golden_compare("clean_reasons", &lines.join("\n"));
}

#[test]
fn golden_clean_summary_lines() {
    setup_no_color();
    let summary = CleanSummary {
        stale_servers: 2,
        orphans: 1,
        zombies: 0,
        idle_listeners: 1,
        listening: 3,
        estimated_bytes: 256 * 1024 * 1024,
    };
    let out = format_summary_lines(&summary, 4);
    golden_compare("clean_summary", &out);
}

#[test]
fn golden_kill_summary() {
    setup_no_color();
    let results = [
        KillResult::new(100 * 1024 * 1024, vec![3000, 5173], KillOutcome::Terminated),
        KillResult::new(50 * 1024 * 1024, vec![8080], KillOutcome::StillAlive),
        KillResult::new(25 * 1024 * 1024, vec![3000], KillOutcome::ForceKilled),
    ];
    let out = format_kill_summary(&results);
    golden_compare("kill_summary", &out);
}

#[test]
fn golden_ports_table() {
    setup_no_color();
    let procs = vec![
        ProcessInfo {
            pid: 100,
            ppid: 1,
            name: "node".into(),
            cpu: 0.0,
            memory_bytes: 0,
            ports: vec![3000],
            command: None,
            cwd: None,
            run_time_secs: 0,
            is_zombie: false,
        },
        ProcessInfo {
            pid: 200,
            ppid: 1,
            name: "python3".into(),
            cpu: 0.0,
            memory_bytes: 0,
            ports: vec![8080],
            command: None,
            cwd: None,
            run_time_secs: 0,
            is_zombie: false,
        },
    ];
    let rows = vec![(8080, 200), (3000, 100)];
    let out = format_ports_table(&rows, &procs);
    golden_compare("ports_table", &out);
}
