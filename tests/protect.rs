use sweeper::process::protect::is_protected;

#[test]
fn protects_system_daemons() {
    assert!(is_protected("kernel_task"));
    assert!(is_protected("launchd"));
    assert!(is_protected("WindowServer"));
}

#[test]
fn allows_dev_processes() {
    assert!(!is_protected("node"));
    assert!(!is_protected("vite"));
}
