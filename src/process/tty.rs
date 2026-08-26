/// Returns true when the process has a controlling TTY (non-zero tty nr).
pub fn has_controlling_tty(pid: u32) -> bool {
    #[cfg(target_os = "linux")]
    {
        linux_controlling_tty(pid)
    }
    #[cfg(target_os = "macos")]
    {
        macos_controlling_tty(pid)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = pid;
        false
    }
}

#[cfg(target_os = "linux")]
fn linux_controlling_tty(pid: u32) -> bool {
    let stat = match std::fs::read_to_string(format!("/proc/{pid}/stat")) {
        Ok(s) => s,
        Err(_) => return false,
    };
    // comm may contain spaces inside parens; tty is the 7th field after ')'.
    let after = match stat.rsplit_once(')') {
        Some((_, rest)) => rest.trim(),
        None => return false,
    };
    let fields: Vec<&str> = after.split_whitespace().collect();
    // fields[0]=state, [1]=ppid, [2]=pgrp, [3]=session, [4]=tty_nr
    fields
        .get(4)
        .and_then(|s| s.parse::<i32>().ok())
        .is_some_and(|tty| tty != 0)
}

#[cfg(target_os = "macos")]
fn macos_controlling_tty(pid: u32) -> bool {
    use std::process::Command;
    let output = match Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "tty="])
        .output()
    {
        Ok(o) => o,
        Err(_) => return false,
    };
    if !output.status.success() {
        return false;
    }
    let tty = String::from_utf8_lossy(&output.stdout);
    let tty = tty.trim();
    !tty.is_empty() && tty != "??"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_shell_has_tty_or_graceful_false() {
        let pid = std::process::id();
        let _ = has_controlling_tty(pid);
    }
}
