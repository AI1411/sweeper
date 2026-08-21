---
title: "[MVP] Task 6: Kill flow (SIGTERM → wait → SIGKILL)"
labels: [enhancement, mvp]
depends_on: [3]
plan: docs/superpowers/plans/2026-08-21-sweeper-mvp.md#task-6
---

## Summary

デフォルト SIGTERM、待機後に必要なら SIGKILL。保護プロセスはスキップ。

## Acceptance criteria

- [ ] `kill_pid(pid, name, force) -> KillOutcome`
- [ ] Outcome: `Terminated` / `ForceKilled` / `StillAlive` / `SkippedProtected`
- [ ] 保護名は kill せず `SkippedProtected`
- [ ] `force=false` では SIGKILL しない
- [ ] `cargo check` 通過

## Files

- Create: `src/process/kill.rs`
- Modify: `src/process/mod.rs`

## References

- Plan Task 6
- Requirements §15–§16
