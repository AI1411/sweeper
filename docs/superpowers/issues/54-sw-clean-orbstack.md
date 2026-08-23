---
title: "[Clean] Integrate OrbStack memory reclaim into sw clean"
labels: [enhancement, post-mvp, orbstack]
depends_on: [47-sw-memory-reclaim, 49-sw-docker]
priority: low
---

## Summary

Extend `sw clean` end-of-session workflow to optionally reclaim OrbStack memory and report combined recovery (memory + disk).

## Motivation

Developers want one "done for the day" command that kills leftovers, releases ports, and reclaims OrbStack RAM — with explicit confirmation for each destructive step.

## Acceptance criteria

- [ ] `sw clean` summary section shows optional OrbStack reclaim estimate when #47 is available
- [ ] User can opt in to reclaim during clean confirm flow (not automatic)
- [ ] Final summary reports: processes killed, ports released, memory reclaimed (`~`), disk recoverable (informational from #49)
- [ ] Each step remains confirmable; reclaim uses #47 safety rules
- [ ] `--json` clean output includes `orbstack_reclaim` block when run
- [ ] No regression to existing process-only clean behavior when OrbStack absent
- [ ] Integration test with mocked reclaim backend

## Example output

```text
Development Cleanup
✓ Kill unused dev processes
✓ Release unused ports
✓ Reclaim OrbStack memory (confirmed)
Recovered
Memory       14.2 GB
Disk          8.7 GB (informational — run sw disk for prune options)
```

## Implementation notes

- `src/commands/clean.rs` — optional post-kill reclaim phase
- Compose #47 + existing clean pipeline

## References

- OrbStack memory requirements §13 (開発環境終了時の一括掃除)
- #47, #49
