---
title: "[Quality] TUI navigation (g/G) and kill preview"
labels: [enhancement, post-mvp]
depends_on: []
priority: high
---

## Summary

Improve TUI **navigation** and **safety**: jump to top/bottom, page up/down, and show a kill preview (PID, ports, command) before sending signals.

## Motivation

Long process lists are common on macOS. Scroll-follow (#47) fixed viewport drift; next quality step is faster movement and **visible context before kill** — core to Sweeper’s safety story.

## Acceptance criteria

### Navigation

- [ ] `g` → first row in filtered list
- [ ] `G` → last row in filtered list
- [ ] `PageUp` / `PageDown` (and optionally `Ctrl-u` / `Ctrl-d`) move by viewport height
- [ ] Cursor + `TableState` stay synced (no regression to scroll-follow fix)
- [ ] Status or title still shows position (e.g. `[42 / 500]`)

### Kill preview

- [ ] Before `k` / `K` / `t` / `T`, show a one-line or two-line preview in status area:
  - process name, PID, ports, truncated command
- [ ] Multi-select: preview count + sample PIDs or “N processes selected”
- [ ] Tree kill: indicate “+ descendants” in preview
- [ ] No silent kill — preview is informational; keys still perform kill (no extra modal unless cheap to add)

### Tests

- [ ] Unit tests for `move_first`, `move_last`, `page_up` / `page_down` cursor bounds
- [ ] Test kill preview string formatting (pure function)

## Suggested UX

```text
Status: Kill preview → node pid 4812 :3000  node ./node_modules/.bin/vite
```

Help footer addition:

```text
[g/G] Jump  [PgUp/PgDn] Page
```

## Implementation notes

- `src/tui/app.rs` — navigation helpers
- `src/tui/mod.rs` — key bindings + preview before `kill_selection`
- `src/tui/ui.rs` — footer help text

## References

- Requirements §16 (safety), §25 (UX: understand before kill)
- `src/tui/{app,mod,ui}.rs`
