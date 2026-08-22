use sweeper::process::protect::is_protected;

#[test]
fn protects_system_daemons() {
    assert!(is_protected("kernel_task"));
    assert!(is_protected("launchd"));
    assert!(is_protected("WindowServer"));
    assert!(is_protected("Finder"));
    assert!(is_protected("Dock"));
}

#[test]
fn protects_case_insensitively() {
    assert!(is_protected("LAUNCHD"));
    assert!(is_protected("windowserver"));
}

#[test]
fn protects_basename_from_path() {
    assert!(is_protected("/sbin/launchd"));
    assert!(is_protected("/System/Library/CoreServices/Finder"));
}

#[test]
fn allows_dev_processes() {
    assert!(!is_protected("node"));
    assert!(!is_protected("vite"));
    assert!(!is_protected("python3"));
}
