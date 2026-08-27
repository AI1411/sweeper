---
title: "[UX] TUI auto-refresh CPU and memory on interval"
labels: [enhancement, post-mvp]
depends_on: [40-process-refresh-strategy]
priority: high
---

## Summary

Periodically refresh CPU and memory in the TUI without requiring manual `r`, while keeping port lookups on a slower cadence.

## Motivation

The TUI tick loop (250ms) only polls input; process stats go stale until the user presses `r`. For monitoring "which dev server is eating CPU", auto-refresh is essential.

## Acceptance criteria

- [ ] Default: refresh CPU/MEM every 2–3 seconds in Processes view
- [ ] Port list refreshed less often (e.g. every 10s) or only on `r` / post-kill
- [ ] Configurable via env `SWEEPER_TUI_REFRESH_SECS` or future config file
- [ ] No full-screen flicker; only changed cells update if feasible
- [ ] Paused while in kill confirm or search mode (optional)
- [ ] Document in README TUI section

## Implementation notes

- `src/tui/mod.rs` — track `last_data_refresh` alongside tick
- `src/process/list.rs` — lightweight CPU/MEM refresh (see #40 / #65)
- Do not call `listening_ports` on every CPU refresh

## References

- `src/tui/mod.rs`
- Related: #74 (smarter process refresh — closed, partial)
