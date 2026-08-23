---
title: "[Docker] sw docker — OrbStack/Docker resource overview"
labels: [enhancement, post-mvp, orbstack]
depends_on: [46-sw-memory]
priority: medium
---

## Summary

Add `sw docker` as a single overview of OrbStack/Docker **memory and disk** footprint with potential recovery estimates.

## Motivation

Developers confuse RAM caches with Docker disk cache. A unified `sw docker` view separates memory vs storage and shows reclaimable totals in one screen.

## Acceptance criteria

- [ ] `sw docker` shows memory section (reuse #46 data): VM total, containers total, reclaimable estimate
- [ ] Disk section: images, containers, volumes, build cache sizes (from `docker system df` or equivalent)
- [ ] **Potential Recovery** footer: memory reclaimable (`~`) + disk reclaimable
- [ ] Clearly label memory vs disk lines (no ambiguous "Docker cache")
- [ ] `--json` output
- [ ] Graceful degradation when Docker daemon unavailable
- [ ] Tests with mocked `docker system df` / stats output

## Example output

```text
$ sw docker
Docker / OrbStack
────────────────────────────────
Memory
OrbStack VM          18.4 GB
Containers            2.5 GB
Reclaimable         ~12.8 GB
Disk
Images                8.4 GB
Containers            1.2 GB
Volumes              12.8 GB
Build Cache          21.4 GB
Potential Recovery
────────────────────────────────
Memory              ~12.8 GB
Disk                 29.7 GB
```

## Implementation notes

- `src/commands/docker.rs`
- Compose #46 memory types + #50 disk types
- Can ship before #50 if disk section uses inline `docker system df` parsing

## References

- OrbStack memory requirements §10–11
- #46, #50
