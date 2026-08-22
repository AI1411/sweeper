---
title: "[Post-MVP] Report estimated memory freed after kill"
labels: [enhancement, post-mvp]
depends_on: []
priority: medium
---

## Summary

After a successful kill (CLI or TUI), show how much memory those processes were using so cleanup feels concrete.

## Motivation

Requirements Priority C (“メモリ解放量推定”). Small UX win that reinforces Sweeper’s “sweep” metaphor.

## Acceptance criteria

- [ ] After confirmed kills, print total RSS (or best-effort memory) freed from the pre-kill snapshot
- [ ] Works for `sw <name>`, `sw :port`, `sw clean`, and TUI multi-select kill
- [ ] Skipped / protected / failed kills are excluded from the total
- [ ] Wording makes clear it is an estimate from the last snapshot (not OS reclaim proof)
- [ ] Tests cover summing memory for successful outcomes only

## Suggested UX

```text
Terminated 3 processes
Estimated memory freed: 812 MB
```

## References

- Requirements §23 Priority C
- `src/commands/{name,port,clean}.rs`, `src/tui/mod.rs`, `ProcessInfo::memory_bytes`
