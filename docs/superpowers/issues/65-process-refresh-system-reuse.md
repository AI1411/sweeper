---
title: "[Performance] Reuse sysinfo System instance for TUI refresh"
labels: [enhancement, post-mvp]
depends_on: []
priority: high
---

## Summary

Keep a persistent `sysinfo::System` in TUI sessions and refresh selectively instead of `System::new_all()` on every `r` and post-kill update.

## Motivation

`refresh_process_list` and `list_processes` each allocate a new `System::new_all()` and refresh all processes. On busy Macs with 500+ processes, manual refresh feels sluggish.

## Acceptance criteria

- [ ] TUI holds one `System` for the session lifetime
- [ ] `r` refresh updates CPU/MEM via selective `ProcessesToUpdate` where possible
- [ ] New PIDs appear; exited PIDs removed (reconciliation)
- [ ] No stale zombie rows after kill
- [ ] Port merge still applied after process refresh
- [ ] Benchmark or documented before/after refresh time in PR
- [ ] CLI cold-start path unchanged (no regression)

## Implementation notes

- `src/process/list.rs` — `ProcessSnapshot` struct wrapping `System`
- `src/tui/app.rs` / `src/tui/mod.rs` — own snapshot
- Follow-up to closed #74

## References

- `src/process/list.rs` (`refresh_process_list`, `list_processes`)
- Related: #74 (closed, not fully implemented)
