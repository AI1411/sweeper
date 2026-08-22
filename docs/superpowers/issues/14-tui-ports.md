---
title: "[Post-MVP] Show listening ports clearly in the TUI"
labels: [enhancement, post-mvp]
depends_on: []
priority: high
---

## Summary

Make listening ports a first-class signal in the TUI process browser so users can see *what* a process is serving before killing it.

## Motivation

Port merge via `lsof` already exists, but the TUI PORT column is often empty/`-` when ports load late or are hard to scan. Port visibility is central to Sweeper’s “stop what you mean” UX.

## Acceptance criteria

- [ ] TUI loads / merges LISTEN ports and shows them in the PORT column (e.g. `:3000,:5173`)
- [ ] Search (`/`) matches port numbers as well as process names
- [ ] Ports appear without requiring a full manual refresh once async merge completes (or refresh is obvious)
- [ ] Sorting or filter by “has port” is optional but welcome
- [ ] No regression to CLI `sw ports` / `sw :port`

## Notes

- Prefer improving the existing async merge path in `src/tui/` rather than blocking startup on `lsof`
- Color for ports already exists in TUI styling — reuse it

## References

- Requirements §23 Priority A (LISTEN port list)
- `src/tui/ui.rs`, `src/tui/mod.rs`, `src/process/ports.rs`
