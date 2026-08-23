use std::sync::{Mutex, OnceLock};
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

type KillHook = Box<dyn Fn(u32, &str, bool) -> Result<KillOutcome> + Send + Sync>;

static KILL_HOOK: OnceLock<Mutex<Option<KillHook>>> = OnceLock::new();

fn hook_slot() -> &'static Mutex<Option<KillHook>> {
    KILL_HOOK.get_or_init(|| Mutex::new(None))
}

/// Test-only hook for integration tests. Clears when `None`.
pub fn set_kill_hook(hook: Option<KillHook>) {
    *hook_slot().lock().expect("kill hook lock") = hook;
}

fn pid_alive(pid: u32) -> bool {
    kill(Pid::from_raw(pid as i32), None).is_ok()
}

fn kill_pid_real(pid: u32, name: &str, force: bool) -> Result<KillOutcome> {
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

pub fn kill_pid(pid: u32, name: &str, force: bool) -> Result<KillOutcome> {
    if let Some(hook) = hook_slot().lock().expect("kill hook lock").as_ref() {
        return hook(pid, name, force);
    }
    kill_pid_real(pid, name, force)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kill_hook_is_used_when_set() {
        set_kill_hook(Some(Box::new(|pid, _name, _force| {
            Ok(if pid == 42 {
                KillOutcome::Terminated
            } else {
                KillOutcome::StillAlive
            })
        })));
        assert_eq!(
            kill_pid(42, "node", false).unwrap(),
            KillOutcome::Terminated
        );
        assert_eq!(kill_pid(1, "node", false).unwrap(), KillOutcome::StillAlive);
        set_kill_hook(None);
    }
}
