---
title: "[Disk] sw disk — Docker and dev storage consumption"
labels: [enhancement, post-mvp, orbstack]
depends_on: []
priority: medium
---

## Summary

Add `sw disk` to inspect Docker images, volumes, build cache, and stopped containers — separate from RAM/page cache (`sw memory`).

## Motivation

"Docker cache" conflates memory and disk. Sweeper must treat **disk** artifacts explicitly so users know what `docker system prune` would affect vs `sw memory reclaim`.

## Acceptance criteria

- [ ] `sw disk` lists Docker disk usage: images, containers (incl. stopped), volumes, build cache
- [ ] Show reclaimable/potential savings per category where `docker system df` provides it
- [ ] Optional: top N largest images (`sw disk --top 10`)
- [ ] **No destructive prune** in this issue — display and estimates only; prune flows are separate (future)
- [ ] Distinguish from memory caches in help text and output headers
- [ ] `--json` output
- [ ] Works when Docker is available (macOS OrbStack / Linux Docker)
- [ ] Tests with fixture `docker system df -v` output

## Example output

```text
$ sw disk
Disk (Docker)
────────────────────────────────
Images                8.4 GB  (reclaimable 3.1 GB)
Containers            1.2 GB  (reclaimable 0.8 GB)
Volumes              12.8 GB
Build Cache          21.4 GB  (reclaimable 18.2 GB)
```

## Implementation notes

- `src/disk/` or `src/docker/disk.rs`
- `src/cli.rs` — `Disk` subcommand
- Used by #49 `sw docker` disk section

## References

- OrbStack memory requirements §10
- #49
