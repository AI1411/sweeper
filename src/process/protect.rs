const PROTECTED: &[&str] = &[
    "kernel_task",
    "launchd",
    "WindowServer",
    "loginwindow",
    "SystemUIServer",
    "Finder",
    "Dock",
];

pub fn is_protected(name: &str) -> bool {
    let base = name.rsplit('/').next().unwrap_or(name);
    PROTECTED.iter().any(|p| base.eq_ignore_ascii_case(p))
}
