---
title: "[Memory] sw memory — OrbStack/Docker memory analysis"
labels: [enhancement, post-mvp, orbstack]
depends_on: []
priority: high
---

## Summary

Add `sw memory` to analyze macOS system memory, OrbStack VM usage, and per-container Docker memory — with unattributed gap reporting.

## Motivation

OrbStack can show large memory use in Activity Monitor that does not map 1:1 to running containers (page cache, filesystem cache, VM overhead, background processes). Sweeper should explain **what** OrbStack memory is used for, not only a single total.

## Acceptance criteria

- [ ] `sw memory` prints system summary: total, used, available
- [ ] OrbStack VM total memory (when OrbStack/Docker is present)
- [ ] Per-container table: name, memory, status; default sort by memory descending
- [ ] `sw memory --sort memory|name|status` (at minimum `--sort memory`)
- [ ] Compute **Unattributed** = VM memory − container total; show when gap is significant
- [ ] List **possible causes** for unattributed memory (page cache, filesystem cache, VM memory, background processes) without claiming precision — label estimates with `~` where applicable
- [ ] Do **not** flag non-Docker processes (e.g. standalone `postgres`/`redis` on macOS) as OrbStack containers
- [ ] `--json` output for scripting (reuse global `--json` flag)
- [ ] macOS-only for OrbStack paths; graceful message when OrbStack/Docker unavailable
- [ ] Tests with mocked OrbStack/Docker CLI output fixtures

## Example output

```text
$ sw memory
Memory
────────────────────────────────
System
Total Memory        128 GB
Used                 42 GB
Available             86 GB
OrbStack
OrbStack VM          18.4 GB
Containers
postgres              1.2 GB
redis                420 MB
api                  850 MB
────────────────────────────────
Container Total       2.5 GB
Unattributed         15.9 GB
⚠ Large amount of memory is not attributed to running containers.
Possible causes:
• Linux page cache
• Filesystem cache
• VM memory
• Background processes
```

## Implementation notes

- New module e.g. `src/memory/` or `src/orbstack/`
- Data sources: `docker stats`, OrbStack CLI/API (investigate available interfaces on macOS)
- `src/cli.rs` — add `Memory { sort: Option<SortField> }` subcommand
- Distinguish **memory** caches from **disk** Docker artifacts (see #49)

## References

- OrbStack memory requirements (2026-08)
- `src/cli.rs`, `src/json_output.rs`
