---
title: "[UX] Developer-centric default sort in TUI and lists"
labels: [enhancement, post-mvp]
depends_on: []
priority: low
---

## Summary

Change default process ordering from name-sorted to a developer-friendly order: listeners first, then higher memory, then CPU.

## Motivation

Alphabetical sort buries the processes users care about (node on :3000) below unrelated entries. Port-first ordering matches Sweeper’s mission.

## Acceptance criteria

- [ ] TUI initial list and post-refresh use new sort (document order)
- [ ] Optional: preserve user toggle to sort by name (`o` for order?)
- [ ] CLI `sw` bare TUI and `list_processes` consumers reviewed for consistency
- [ ] Search/filter does not break sort stability
- [ ] Unit test for sort key function

## Suggested order

1. Has LISTEN port (yes first)
2. Memory descending
3. CPU descending
4. Name ascending (tie-break)

## Implementation notes

- `src/process/list.rs` — `sort_processes_for_display`
- `src/tui/app.rs` — apply after `refilter` or before filter

## References

- Requirements §6.2, §25
- `src/process/list.rs`, `src/tui/app.rs`
