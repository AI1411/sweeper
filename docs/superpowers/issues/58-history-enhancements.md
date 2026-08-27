---
title: "[History] Project metadata, filters, and quick re-watch"
labels: [enhancement, post-mvp]
depends_on: []
priority: low
---

## Summary

Enrich kill history with project context and add CLI filters plus shortcuts to re-watch recently freed ports.

## Motivation

History currently stores PID, name, ports, and signal but not project name. Developers want to audit "what did I kill for my-app?" and quickly verify a port is free after cleanup.

## Acceptance criteria

- [ ] `HistoryEntry` stores optional `project` field (backward-compatible JSON migration)
- [ ] `sw history --project <name>` filters entries
- [ ] `sw history --since 1h` or `--last N` (extend existing `--last`)
- [ ] `sw history --json` includes new fields
- [ ] Optional: `sw watch :3000` suggests recently killed ports from history (document only, or `--from-history`)
- [ ] Tests for load/save with legacy entries missing `project`

## Implementation notes

- `src/history/mod.rs` — schema extension
- `src/commands/history.rs` — filter flags
- Append project name in kill flows (`src/commands/port.rs`, `src/tui/mod.rs`, `src/commands/clean.rs`)

## References

- `src/history/mod.rs`, `src/commands/history.rs`
