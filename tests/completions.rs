use std::process::Command;

#[test]
fn bash_completion_contains_subcommands() {
    let output = Command::new(env!("CARGO_BIN_EXE_sw"))
        .args(["completions", "bash"])
        .output()
        .expect("run sw completions bash");
    assert!(output.status.success(), "completions failed");
    let script = String::from_utf8_lossy(&output.stdout);
    assert!(!script.is_empty());
    for cmd in [
        "ports",
        "top",
        "clean",
        "history",
        "project",
        "memory",
        "docker",
        "disk",
        "cache",
        "doctor",
        "completions",
    ] {
        assert!(
            script.contains(cmd),
            "missing subcommand {cmd} in bash completion"
        );
    }
    for flag in ["--force", "--tree", "--dry-run", "--json"] {
        assert!(
            script.contains(flag),
            "missing flag {flag} in bash completion"
        );
    }
}

#[test]
fn zsh_completion_non_empty() {
    let output = Command::new(env!("CARGO_BIN_EXE_sw"))
        .args(["completions", "zsh"])
        .output()
        .expect("run sw completions zsh");
    assert!(output.status.success());
    assert!(!output.stdout.is_empty());
}

#[test]
fn rejects_unknown_shell() {
    let output = Command::new(env!("CARGO_BIN_EXE_sw"))
        .args(["completions", "powershell"])
        .output()
        .expect("run sw completions powershell");
    assert!(!output.status.success());
}
