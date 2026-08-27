---
title: "[UX] TUI sort toggle (name / CPU / memory / port)"
labels: [enhancement, post-mvp]
depends_on: [42-default-sort]
priority: medium
---

## Summary

Let users switch process sort order in the TUI without leaving the browser.

## Motivation

Default sort (listeners → memory → CPU → name) is developer-friendly, but sometimes users want pure CPU or memory order (like `sw top`).

## Acceptance criteria

- [ ] `s` cycles sort modes: default / CPU desc / memory desc / name asc / port first
- [ ] Current sort shown in status bar or table title
- [ ] Sort persists for session (not saved to disk in v1)
- [ ] Search/filter preserves relative order within filtered set
- [ ] Unit test for sort key function per mode

## Implementation notes

- `src/tui/app.rs` — `SortMode` enum, apply in `refilter`
- `src/process/list.rs` — extract sort functions from `sort_processes_for_display`

## References

- `src/process/list.rs`, `src/tui/app.rs`
- Related: #76 (default sort — implemented)
