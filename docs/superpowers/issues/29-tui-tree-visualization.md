---
title: "[UX] TUI process tree visualization"
labels: [enhancement, post-mvp]
depends_on: []
priority: low
---

## Summary

Visualize parent-child relationships in the TUI (indent tree or expandable nodes), not only tree kill via `t` / `T`.

## Motivation

Requirements §10 show tree diagrams. Users need to see why tree kill matters (node → next-server → workers) before using `t`.

## Acceptance criteria

- [ ] Toggle or sub-view shows children under parent rows (indent or tree chars `└─`)
- [ ] Tree built from PPID snapshot in process list
- [ ] Cursor can move on flat list or tree-expanded list (document behavior)
- [ ] Tree kill (`t` / `T`) still works on selection
- [ ] Performance acceptable for ~500 processes (no O(n²) per frame)
- [ ] Unit tests for tree layout from fixture `ProcessInfo` list

## Implementation notes

- `src/process/tree.rs` — layout helper
- `src/tui/ui.rs` — render indented rows

## References

- Requirements §10 (プロセスツリー)
- `src/process/tree.rs`, `src/tui/ui.rs`
