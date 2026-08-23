---
title: "[Memory] High memory usage and growth warnings"
labels: [enhancement, post-mvp, orbstack]
depends_on: [46-sw-memory]
priority: medium
---

## Summary

Detect and warn on Docker containers using unusually high memory; lay groundwork for future growth/leak detection.

## Motivation

A single container consuming multiple GB should stand out in `sw memory` output. Future leak detection needs baseline snapshots.

## Acceptance criteria

- [ ] `sw memory` flags containers above configurable threshold (default e.g. 2 GB) with `⚠ High Memory Usage`
- [ ] Show container name and current memory in warning block
- [ ] Threshold override: `sw memory --warn-above <bytes|GB>` or env `SWEEPER_MEMORY_WARN_GB`
- [ ] Optional `--json`: include `warnings[]` with `{ container, memory_bytes, kind: "high_usage" }`
- [ ] **Phase 1 only:** static threshold warnings (no time-series yet)
- [ ] Document future `memory watch` / leak detection in issue #51–#52 (do not implement time-series here)
- [ ] Tests with fixture container stats

## Example output

```text
⚠ High Memory Usage
api                  4.8 GB
Memory usage exceeds warning threshold (2.0 GB).
```

## Implementation notes

- Extend `sw memory` command from #46
- Store no persistent history in this issue (defer to #52)

## References

- OrbStack memory requirements §6, §13
- #46, #51, #52
