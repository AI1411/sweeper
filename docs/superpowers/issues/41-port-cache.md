---
title: "[Performance] Port lookup cache with TTL"
labels: [enhancement, post-mvp]
depends_on: []
priority: low
---

## Summary

Cache `listening_ports()` results with a short TTL to avoid repeated `lsof` invocations during TUI session and rapid CLI calls.

## Motivation

TUI spawns port loader on start and on `r`; kill flow may call `listening_ports()` again. `lsof` on full LISTEN table is expensive.

## Acceptance criteria

- [ ] Shared cache module with configurable TTL (default e.g. 2–5s)
- [ ] TUI `r` can force refresh (bypass cache)
- [ ] CLI commands get fresh or cached ports per flag (`--no-cache` optional)
- [ ] Thread-safe if port loader thread shares cache
- [ ] Unit test: second call within TTL does not spawn `lsof` (mock Command)

## Implementation notes

- `src/process/ports.rs` or `src/cache.rs`
- Complements native port resolution (#31)

## References

- `src/tui/mod.rs` (`spawn_port_loader`), `src/process/ports.rs`
