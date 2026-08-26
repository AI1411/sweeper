use std::process::Command;

use sweeper::commands::doctor::{collect_checks, CheckStatus};

#[test]
fn collect_checks_includes_core_items() {
    let checks = collect_checks();
    let names: Vec<_> = checks.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains(&"sw binary"));
    assert!(names.contains(&"native port lookup"));
    assert!(names.contains(&"history writable"));
    assert!(names.contains(&"protect config readable"));
}

#[test]
fn doctor_cli_runs_successfully() {
    std::env::set_var("NO_COLOR", "1");
    let output = Command::new(env!("CARGO_BIN_EXE_sw"))
        .arg("doctor")
        .output()
        .expect("run sw doctor");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() || output.status.code() == Some(1),
        "doctor failed: stdout={stdout} stderr={stderr}"
    );
    assert!(stdout.contains("Sweeper doctor") || stdout.contains("checks"));
}

#[test]
fn doctor_json_output() {
    std::env::set_var("NO_COLOR", "1");
    let output = Command::new(env!("CARGO_BIN_EXE_sw"))
        .args(["doctor", "--json"])
        .output()
        .expect("run sw doctor --json");
    assert!(output.status.success() || output.status.code() == Some(1));
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("doctor json parse");
    let checks = json["checks"].as_array().expect("checks array");
    assert!(!checks.is_empty());
    assert!(checks[0]["name"].is_string());
    assert!(checks[0]["status"].is_string());
    assert!(checks[0]["message"].is_string());
}

#[test]
fn check_status_serialization() {
    assert_eq!(
        serde_json::to_string(&CheckStatus::Pass).unwrap(),
        "\"pass\""
    );
}
