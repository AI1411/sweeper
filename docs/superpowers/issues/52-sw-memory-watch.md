---
title: "[Memory] sw memory watch — real-time OrbStack monitoring"
labels: [enhancement, post-mvp, orbstack]
depends_on: [46-sw-memory]
priority: low
---

## Summary

Add `sw memory watch` to stream OrbStack VM memory at a fixed interval with delta highlighting.

## Motivation

Developers need to see whether memory is growing during builds or container restarts without opening Activity Monitor.

## Acceptance criteria

- [ ] `sw memory watch` refreshes OrbStack VM total on interval (default 5s, `--interval <secs>`)
- [ ] Show timestamped lines and short-term delta (e.g. `↑ +1.1 GB / 15 sec`)
- [ ] Optional `--containers` includes per-container deltas
- [ ] Clean exit on `Ctrl+C`
- [ ] `--json` emits one JSON object per tick (newline-delimited) for scripting
- [ ] macOS + OrbStack only
- [ ] Tests for delta calculation logic (unit tests; no live watch in CI)

## Example output

```text
OrbStack
10:31:01     4.2 GB
10:31:05     4.4 GB
10:31:10     4.8 GB  ↑ +0.6 GB / 9 sec
```

## Implementation notes

- Subcommand or flag on `sw memory`
- Builds on #46 data collection

## References

- OrbStack memory requirements §13 (メモリ監視)
