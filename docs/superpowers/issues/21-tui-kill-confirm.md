---
title: "[Safety] TUI kill confirmation before sending signals"
labels: [enhancement, post-mvp]
depends_on: []
priority: high
---

## Summary

Add an explicit confirmation step in the TUI before `k` / `K` / `t` / `T` send signals. Kill preview is informational today; users still kill immediately on keypress.

## Motivation

Sweeper’s safety story is “understand before kill.” CLI paths use `confirm()`; TUI is weaker. A one-step `y/N` (or `k` → preview → `y`) reduces accidental kills on long process lists.

## Acceptance criteria

- [ ] After kill keypress, show preview and require confirmation before any signal is sent
- [ ] Works for single row, multi-select, and tree kill (`t` / `T`)
- [ ] `Esc` or `n` cancels without killing
- [ ] Protected processes still blocked; no confirmation bypass
- [ ] Footer/help documents the confirmation flow
- [ ] Unit tests for confirmation state machine (pure logic in `app.rs` or helper)

## Suggested UX

```text
Kill preview → node pid 4812 :3000  node ./node_modules/.bin/vite (+ descendants)
Confirm kill? [y/N]
```

## Implementation notes

- Extend `preview_and_kill` in `src/tui/mod.rs` or add `App::confirming_kill` state
- Reuse `format_kill_preview` from `src/tui/app.rs`
- Avoid extra modal complexity if possible; status-line prompt is enough

## References

- Requirements §16 (安全機能), §25 (UX: understand before kill)
- `src/tui/mod.rs`, `src/tui/app.rs`, `src/commands/confirm.rs`
