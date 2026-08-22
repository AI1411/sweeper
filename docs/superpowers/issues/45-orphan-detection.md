---
title: "[Clean] Cross-project orphan and stale parent detection"
labels: [enhancement, post-mvp]
depends_on: []
priority: medium
---

## Summary

Improve orphan detection when parent shell died but children remain (vite, node workers), including cross-project edge cases.

## Motivation

Classic dev leftover: terminal closed, child dev server keeps listening. Current orphan rules may miss ppid edge cases or zombie parent PIDs.

## Acceptance criteria

- [ ] Detect children whose PPID is not in snapshot or parent is `<defunct>` / zombie
- [ ] Flag listeners with parent not in user session tree (optional heuristic)
- [ ] Human-readable reason: `orphan-parent (ppid 9999 missing)` per #49 style
- [ ] Tests: fixture snapshot with missing parent PID, zombie parent, nested workers
- [ ] Protected processes excluded

## Implementation notes

- `src/clean/mod.rs` — extend orphan proposal logic
- `src/process/tree.rs` — optional session root detection

## References

- Requirements §10, §14
- `src/clean/mod.rs`, issue #49 reason format
