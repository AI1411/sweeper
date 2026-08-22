---
title: "[Testing] TUI smoke tests with ratatui test backend"
labels: [enhancement, post-mvp]
depends_on: []
priority: medium
---

## Summary

Add automated tests for TUI key handling: navigation, search filter, ports-only toggle, and kill preview strings without a real terminal.

## Motivation

TUI logic in `mod.rs` / `app.rs` is mostly untested at integration level. Regressions in scroll-follow, `g/G`, and preview are easy to reintroduce.

## Acceptance criteria

- [ ] Extract or test `handle_key` logic with injectable key events
- [ ] Tests cover: `g`/`G`, PageUp/Down, search refilter, `p` ports filter, kill preview formatting
- [ ] Optional: ratatui `TestBackend` snapshot of one frame (layout smoke)
- [ ] Tests run in CI without TTY (`cargo test`)
- [ ] No flaky timing-dependent tests

## Implementation notes

- `src/tui/mod.rs` — `handle_key` may need `pub(crate)` for test module or `tests/tui_smoke.rs` with `#[path]`
- Existing unit tests in `src/tui/app.rs` as pattern

## References

- `src/tui/{mod,app,ui}.rs`
- Issue #50 (navigation regression risk)
