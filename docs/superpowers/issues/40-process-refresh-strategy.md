---
title: "[Performance] Smarter process list refresh in TUI"
labels: [enhancement, post-mvp]
depends_on: []
priority: low
---

## Summary

Reduce TUI refresh cost: partial updates for CPU/memory instead of full `refresh_processes` on every `r` and post-kill refresh.

## Motivation

`list_processes()` refreshes all processes via sysinfo — costly on machines with many processes. TUI `r` and post-kill refresh feel sluggish on busy Macs.

## Acceptance criteria

- [ ] Measure baseline refresh time (document in issue or PR)
- [ ] `r` refresh updates CPU/MEM without full process respawn where sysinfo allows
- [ ] New processes appear after refresh (PID list reconciliation)
- [ ] No stale zombie rows after kill (regression test or manual checklist)
- [ ] Port merge still applied after process refresh

## Implementation notes

- `src/process/list.rs` — `refresh_cpu_memory(existing: &mut Vec<ProcessInfo>)`
- `sysinfo::ProcessesToUpdate` selective refresh

## References

- `src/process/list.rs`, `src/tui/app.rs` (`refresh`)
