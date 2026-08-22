---
title: "[Quality] Kill summary: ports released + richer post-kill feedback"
labels: [enhancement, post-mvp]
depends_on: []
priority: high
---

## Summary

After kills (CLI and TUI), show a **complete cleanup summary**: terminated count, estimated memory freed, and **ports released** from the pre-kill snapshot.

## Motivation

Requirements §18 (“Cleanup結果”). Memory estimate exists today, but users still cannot see which dev ports were freed without running `sw ports` again. Port release is central to Sweeper’s “sweep the dev environment” story.

## Acceptance criteria

- [ ] After successful kills, print a unified summary block (CLI + TUI paths)
- [ ] List unique ports that belonged to killed processes (from pre-kill snapshot), e.g. `:3000`, `:5173`
- [ ] Keep / extend estimated memory freed; wording stays “estimate from snapshot”
- [ ] Skipped, protected, and failed kills are excluded from counts and port list
- [ ] Works for `sw <name>`, `sw :port`, `sw clean`, `sw project`, and TUI multi-select / tree kill
- [ ] Empty port list omitted or shown as “none” — no noisy output when no listeners were killed
- [ ] Unit tests for port aggregation from killed `ProcessInfo` rows

## Suggested UX

```text
Terminated 3 process(es) (from last snapshot)
Estimated memory freed: 450 MB

Ports released:
  :3000
  :5173
```

## Implementation notes

- Extend `src/report.rs` (or sibling helper) with `released_ports(killed: &[ProcessInfo]) -> Vec<u16>`
- Share one `print_kill_summary(...)` used by `name`, `port`, `clean`, `project`, and `tui/mod.rs`
- Deduplicate and sort ports numerically

## References

- Requirements §18 (Cleanup結果)
- `src/report.rs`, `src/commands/{name,port,clean,project}.rs`, `src/tui/mod.rs`
