use sweeper::process::protect::is_protected;

#[cfg(target_os = "macos")]
#[test]
fn protects_macos_system_daemons() {
    assert!(is_protected("kernel_task"));
    assert!(is_protected("launchd"));
    assert!(is_protected("WindowServer"));
    assert!(is_protected("Finder"));
    assert!(is_protected("Dock"));
    assert!(is_protected("coreaudiod"));
    assert!(is_protected("mds"));
    assert!(is_protected("bluetoothd"));
}

#[cfg(target_os = "linux")]
#[test]
fn protects_linux_system_daemons() {
    assert!(is_protected("systemd"));
    assert!(is_protected("sshd"));
    assert!(is_protected("dbus-daemon"));
    assert!(is_protected("NetworkManager"));
    assert!(is_protected("systemd-journald"));
}

#[test]
fn protects_case_insensitively() {
    #[cfg(target_os = "macos")]
    {
        assert!(is_protected("LAUNCHD"));
        assert!(is_protected("windowserver"));
    }
    #[cfg(target_os = "linux")]
    {
        assert!(is_protected("SYSTEMD"));
        assert!(is_protected("SSHD"));
    }
}

#[test]
fn protects_basename_from_path() {
    #[cfg(target_os = "macos")]
    {
        assert!(is_protected("/sbin/launchd"));
        assert!(is_protected("/System/Library/CoreServices/Finder"));
    }
    #[cfg(target_os = "linux")]
    {
        assert!(is_protected("/usr/sbin/sshd"));
        assert!(is_protected("/lib/systemd/systemd"));
    }
}

#[test]
fn allows_dev_processes() {
    assert!(!is_protected("node"));
    assert!(!is_protected("vite"));
    assert!(!is_protected("python3"));
}
