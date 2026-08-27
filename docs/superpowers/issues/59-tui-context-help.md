---
title: "[UX] TUI context-sensitive help footer"
labels: [enhancement, post-mvp]
depends_on: []
priority: high
---

## Summary

Show view-specific keybindings in the TUI footer instead of listing all 20+ keys on every screen.

## Motivation

The footer currently displays every binding (Move, Kill, Tree, Projects, Clean, OrbStack, …) regardless of view mode. On Clean or Project views, most keys are irrelevant and the help bar is crowded.

## Acceptance criteria

- [ ] Footer help changes by view: Processes / Projects / Clean / OrbStack / Detail / Search
- [ ] `?` toggles a full keybinding overlay (all keys, dismiss with Esc)
- [ ] Status line remains visible below help
- [ ] README documents `?` overlay
- [ ] TUI smoke test covers at least one view-specific footer string

## Suggested UX

Processes view footer:
```text
[↑↓] Move  [Space] Select  [k] Kill→y  [p] Ports  [P] Projects  [c] Clean  [?] All keys
```

Clean view footer:
```text
[↑↓] Move  [Space] Select  [k] Kill→y  [H] High-only  [c] Back  [?] All keys
```

## Implementation notes

- `src/tui/ui.rs` — `draw_footer` branches on `app.view_mode`, `app.resources_open`, `app.searching`
- `src/tui/app.rs` — optional `show_help_overlay: bool`

## References

- `src/tui/ui.rs` (`draw_footer`)
