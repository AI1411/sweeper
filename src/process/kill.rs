use std::thread;
use std::time::Duration;

use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

use crate::error::{Result, SweeperError};
use crate::process::protect::is_protected;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KillOutcome {
    Terminated,
    ForceKilled,
    StillAlive,
    SkippedProtected,
}

fn pid_alive(pid: u32) -> bool {
    // signal 0 checks existence
    kill(Pid::from_raw(pid as i32), None).is_ok()
}

pub fn kill_pid(pid: u32, name: &str, force: bool) -> Result<KillOutcome> {
    if is_protected(name) {
        return Ok(KillOutcome::SkippedProtected);
    }

    let raw = Pid::from_raw(pid as i32);
    kill(raw, Signal::SIGTERM).map_err(|e| SweeperError::Kill(pid, e.to_string()))?;

    thread::sleep(Duration::from_secs(2));

    if !pid_alive(pid) {
        return Ok(KillOutcome::Terminated);
    }

    if force {
        kill(raw, Signal::SIGKILL).map_err(|e| SweeperError::Kill(pid, e.to_string()))?;
        thread::sleep(Duration::from_millis(200));
        if !pid_alive(pid) {
            return Ok(KillOutcome::ForceKilled);
        }
        return Ok(KillOutcome::StillAlive);
    }

    Ok(KillOutcome::StillAlive)
}
