---
title: "[Performance] Parallel SIGTERM wait for batch kills"
labels: [enhancement, post-mvp]
depends_on: []
priority: low
---

## Summary

Send SIGTERM to all targets, wait once, then verify — instead of sequential 2s sleep per PID.

## Motivation

`kill_pid` sleeps 2 seconds after each SIGTERM. Killing 10 processes sequentially can take 20+ seconds even when all exit immediately.

## Acceptance criteria

- [ ] Batch kill API: send SIGTERM to all PIDs, single wait (configurable, default 2s), then check each
- [ ] Force-kill prompt still per-process or batched (document choice)
- [ ] Protected processes skipped as today
- [ ] TUI multi-select kill uses batch path
- [ ] `--dry-run` unchanged
- [ ] Tests with mock kill hook

## Implementation notes

- `src/process/kill.rs` — `kill_pids_batch(pids, force)`
- `src/tui/mod.rs`, `src/commands/port.rs`, `src/commands/clean.rs` — call batch API

## References

- `src/process/kill.rs` (`kill_pid`, 2s sleep)
