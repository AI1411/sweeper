---
title: "[Cache] sw cache — dev tool cache inspection (future)"
labels: [enhancement, post-mvp]
depends_on: [50-sw-disk]
priority: low
---

## Summary

Add `sw cache` to discover and report safe-to-clear caches from common dev tools (npm, cargo, Homebrew, etc.) — distinct from OrbStack memory and Docker disk.

## Motivation

Sweeper's roadmap positions Cache as a first-class resource alongside Process, Port, Memory, Docker, and Disk. Developers need one place to see **dev tool** cache sizes without conflating them with Linux page cache or Docker build cache.

## Acceptance criteria

- [ ] `sw cache` lists known cache locations with estimated sizes (best-effort; `~` when approximate)
- [ ] Categories: package managers (npm/pnpm/yarn, cargo), build tools (optional), others behind feature flags
- [ ] **Display only** in v1 — no auto-clean; future prune commands require confirmation
- [ ] Clearly separate from `sw memory` (RAM) and `sw disk` (Docker storage) in headers and docs
- [ ] `--json` output
- [ ] macOS paths first; Linux/XDG where applicable
- [ ] Tests with temp-dir fixtures

## Example output

```text
$ sw cache
Dev Caches
────────────────────────────────
npm                  4.2 GB   ~/.npm
cargo                8.1 GB   ~/.cargo/registry
pnpm                 1.3 GB   ~/Library/Caches/pnpm
```

## Implementation notes

- `src/cache/mod.rs` — pluggable `CacheProvider` trait per tool
- Start with 2–3 high-value providers; expand incrementally
- Do not overlap Docker build cache (#50)

## References

- OrbStack memory requirements §2, §14 (Cache pillar)
- #50
