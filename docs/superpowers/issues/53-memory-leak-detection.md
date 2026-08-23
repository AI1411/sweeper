---
title: "[Memory] Container memory leak candidate detection"
labels: [enhancement, post-mvp, orbstack]
depends_on: [52-sw-memory-watch]
priority: low
---

## Summary

Detect containers whose memory grows monotonically over a observation window and flag possible leaks.

## Motivation

Long-running API containers that grow from hundreds of MB to several GB over 30–60 minutes are common dev leftovers; Sweeper should surface them proactively.

## Acceptance criteria

- [ ] Persist lightweight snapshots (container → memory_bytes + timestamp) under app data dir (XDG / macOS Application Support)
- [ ] `sw memory` (or `sw memory --leaks`) lists **Possible Memory Leak** candidates when growth exceeds threshold over window (e.g. +1 GB in 30 min, configurable)
- [ ] Show start memory, current memory, growth, elapsed time
- [ ] Snapshots are opt-out via env or flag; document retention (e.g. 24h rolling)
- [ ] No auto-stop of containers — display only (align with Sweeper safety model)
- [ ] `--json` includes leak candidates
- [ ] Unit tests for growth detection algorithm on fixture time series

## Example output

```text
⚠ Possible Memory Leak
container api
30 min ago       820 MB
now              3.8 GB
Growth          +2.98 GB
```

## Implementation notes

- Depends on watch/history infrastructure from #52
- `src/memory/history.rs` or extend existing `src/history/`

## References

- OrbStack memory requirements §6, §13
- #46, #52
