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

const BATCH_TERM_WAIT: Duration = Duration::from_secs(2);

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
    let results = kill_pids_batch(&[(pid, name)], force)?;
    Ok(results
        .into_iter()
        .next()
        .map(|(_, o)| o)
        .unwrap_or(KillOutcome::StillAlive))
}

pub fn kill_pid(pid: u32, name: &str, force: bool) -> Result<KillOutcome> {
    if let Some(hook) = hook_slot().lock().expect("kill hook lock").as_ref() {
        return hook(pid, name, force);
    }
    kill_pid_real(pid, name, force)
}

/// Send SIGTERM to all targets, wait once, then verify each PID.
pub fn kill_pids_batch(targets: &[(u32, &str)], force: bool) -> Result<Vec<(u32, KillOutcome)>> {
    if let Some(hook) = hook_slot().lock().expect("kill hook lock").as_ref() {
        return targets
            .iter()
            .map(|(pid, name)| Ok((*pid, hook(*pid, name, force)?)))
            .collect();
    }

    let mut outcomes = Vec::with_capacity(targets.len());
    let mut pending: Vec<(u32, Pid)> = Vec::new();

    for (pid, name) in targets {
        if is_protected(name) {
            outcomes.push((*pid, KillOutcome::SkippedProtected));
            continue;
        }
        let raw = Pid::from_raw(*pid as i32);
        match kill(raw, Signal::SIGTERM) {
            Ok(()) => pending.push((*pid, raw)),
            Err(e) => return Err(SweeperError::Kill(*pid, e.to_string())),
        }
    }

    if !pending.is_empty() {
        thread::sleep(BATCH_TERM_WAIT);
    }

    for (pid, raw) in pending {
        if !pid_alive(pid) {
            outcomes.push((pid, KillOutcome::Terminated));
            continue;
        }
        if force {
            kill(raw, Signal::SIGKILL).map_err(|e| SweeperError::Kill(pid, e.to_string()))?;
            thread::sleep(Duration::from_millis(200));
            if !pid_alive(pid) {
                outcomes.push((pid, KillOutcome::ForceKilled));
            } else {
                outcomes.push((pid, KillOutcome::StillAlive));
            }
        } else {
            outcomes.push((pid, KillOutcome::StillAlive));
        }
    }

    Ok(outcomes)
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

    #[test]
    fn batch_kill_uses_hook_for_each_pid() {
        set_kill_hook(Some(Box::new(|pid, _name, _force| {
            Ok(if pid == 10 || pid == 20 {
                KillOutcome::Terminated
            } else {
                KillOutcome::StillAlive
            })
        })));
        let results = kill_pids_batch(&[(10, "a"), (20, "b"), (30, "c")], false).unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0], (10, KillOutcome::Terminated));
        assert_eq!(results[1], (20, KillOutcome::Terminated));
        assert_eq!(results[2], (30, KillOutcome::StillAlive));
        set_kill_hook(None);
    }
}
