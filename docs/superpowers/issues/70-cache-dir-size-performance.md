---
title: "[Performance] Faster sw cache directory size calculation"
labels: [enhancement, post-mvp]
depends_on: [55-sw-cache]
priority: low
---

## Summary

Speed up `sw cache` size estimation for large directories (cargo registry, npm cache) without blocking the CLI for minutes.

## Motivation

`dir_size` recursively walks every file synchronously. A multi-GB cargo registry makes `sw cache` feel hung on first run.

## Acceptance criteria

- [ ] Use platform `du` when available for approximate totals (with `~` label)
- [ ] Or: cap walk depth / sample + cache results to disk with TTL
- [ ] Progress hint for slow scans (`Scanning cargo registry…`)
- [ ] `--json` includes `approximate: true` and `scan_duration_ms`
- [ ] Tests use temp dirs; mock `du` optional

## Implementation notes

- `src/cache/mod.rs` — `dir_size`, `cache_entry`
- Cache scan results under app data dir

## References

- `src/cache/mod.rs`
- Related: #116 (`sw cache` — implemented)
