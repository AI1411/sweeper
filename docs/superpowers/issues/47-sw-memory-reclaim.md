---
title: "[Memory] sw memory reclaim — safe OrbStack memory reclaim"
labels: [enhancement, post-mvp, orbstack]
depends_on: [46-sw-memory]
priority: high
---

## Summary

Add `sw memory reclaim` to estimate reclaimable OrbStack/Linux VM memory and perform safe reclaim operations only after user confirmation.

## Motivation

Developers need a safe way to free OrbStack VM memory (page cache, filesystem cache) without guessing shell commands. Sweeper must analyze first, confirm, then report before/after.

## Acceptance criteria

- [ ] `sw memory reclaim` shows pre-flight analysis: VM memory, container total, estimated reclaimable (`~` prefix for estimates)
- [ ] Break down estimated sources: Linux page cache, filesystem cache, other (estimates only)
- [ ] Interactive confirm: `Reclaim approximately 12.8 GB? [y/N]` — default **No**
- [ ] Post-run summary: before GB, after GB, recovered GB, success/failure message
- [ ] **Never** auto-reclaim; no reclaim without explicit confirmation
- [ ] Classify actions in docs/help:
  - **Safe:** analyze, display, detect cache, show reclaimable
  - **Caution:** drop caches, restart VM/OrbStack, stop containers — all require confirmation
- [ ] `--dry-run` shows planned actions without executing
- [ ] `--json` mode for scripting (proposal + result structs)
- [ ] macOS + OrbStack only; clear error when unavailable
- [ ] Integration tests with mocked reclaim backend

## Example output

```text
$ sw memory reclaim
OrbStack Memory
────────────────────────────────
VM Memory           18.4 GB
Containers           2.5 GB
Estimated
Reclaimable         ~12.8 GB
Possible sources:
Linux page cache     ~8.2 GB
Filesystem cache     ~3.1 GB
Other                ~1.5 GB

Reclaim approximately 12.8 GB? [y/N] y
Reclaiming memory...
Before              18.4 GB
After                6.1 GB
────────────────────────────────
Recovered           12.3 GB
✓ Memory reclaimed successfully
```

## Implementation notes

- Depends on #46 memory analysis types
- `src/commands/memory.rs` — `reclaim` subcommand or flag
- Backend trait for testability (`MemoryBackend` / mock)
- Document exact reclaim mechanism once OrbStack API is chosen (e.g. `sync; echo 3 > /proc/sys/vm/drop_caches` inside VM, or OrbStack-specific command)

## References

- OrbStack memory requirements §7–9
- `src/commands/confirm.rs` for confirmation patterns
