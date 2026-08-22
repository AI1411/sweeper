---
title: "[Safety] Expand protected process list and user config"
labels: [enhancement, post-mvp]
depends_on: []
priority: high
---

## Summary

Grow the built-in protected process list beyond the current 8 macOS names, and allow users to extend protection via a config file.

## Motivation

`src/process/protect.rs` only blocks a minimal set (`kernel_task`, `launchd`, `WindowServer`, etc.). System services like `mds`, `coreaudiod`, and user-critical tools are not covered. User-defined protection reduces fear of `sw clean` and TUI bulk kill.

## Acceptance criteria

- [ ] Expand default `PROTECTED` list with common macOS system daemons (document each addition)
- [ ] Optional config file, e.g. `~/.config/sweeper/protect.toml` or `~/Library/Application Support/sweeper/protect.toml`
- [ ] Config entries match by process name (case-insensitive), same as built-in list
- [ ] `is_protected` used consistently in CLI kill, TUI kill, and `sw clean` proposals
- [ ] Unit tests for built-in + config merge
- [ ] README section on customizing protection

## Implementation notes

- `src/process/protect.rs` — load config at startup or on first check
- Use `directories` crate for config path (mirror `history` pattern)
- Keep MVP rule: no `-y` / skip confirmations

## References

- Requirements §16 (安全機能)
- `src/process/protect.rs`, `src/history/mod.rs`
