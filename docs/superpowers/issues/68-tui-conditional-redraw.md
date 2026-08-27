---
title: "[Performance] TUI conditional redraw on data change"
labels: [enhancement, post-mvp]
depends_on: [60-tui-auto-refresh-cpu-mem]
priority: medium
---

## Summary

Skip full terminal redraws when neither input nor process data changed since the last frame.

## Motivation

The TUI main loop calls `terminal.draw()` every 250ms even when idle. Reducing unnecessary redraws lowers CPU usage and flicker, especially with auto-refresh enabled.

## Acceptance criteria

- [ ] Track `dirty` flag: set on key input, data refresh, port load, view change
- [ ] Skip `draw()` when not dirty and no input pending
- [ ] Always draw on first frame and after resize (if detectable)
- [ ] Kill confirm preview still draws immediately
- [ ] Manual test: idle TUI uses less CPU (document in PR)

## Implementation notes

- `src/tui/mod.rs` — dirty flag in main loop
- `src/tui/app.rs` — mark dirty on state changes

## References

- `src/tui/mod.rs` (main loop)
