---
title: "[TUI] OrbStack memory and Docker resource dashboard"
labels: [enhancement, post-mvp, orbstack]
depends_on: [46-sw-memory, 49-sw-docker]
priority: medium
---

## Summary

Extend the Sweeper TUI with OrbStack/Docker resource summary and a detail screen with reclaim entry point.

## Motivation

`sw` should evolve from process browser to **macOS dev resource manager**. OrbStack memory and Docker disk belong on the home dashboard.

## Acceptance criteria

- [ ] TUI home shows OrbStack memory total and reclaimable estimate (when available)
- [ ] TUI home shows Docker disk total and recoverable estimate (when #49/#50 data available)
- [ ] Selecting OrbStack opens detail view: memory breakdown, container list, cache estimate
- [ ] Detail keys: `[R]` trigger reclaim flow (delegates to #47 confirm UX), `[C]` containers, `[D]` docker overview, `[Esc]` back
- [ ] Reclaim in TUI uses same confirmation rules as CLI (never silent)
- [ ] Hide OrbStack section when OrbStack/Docker not detected
- [ ] TUI smoke tests for navigation (mock backend)

## Example (home)

```text
┌─────────────────────────────────────────────┐
│ Sweeper                                     │
├─────────────────────────────────────────────┤
│ Processes        183 running                │
│ Ports            12 listening               │
│ OrbStack         18.4 GB  (~12.8 GB reclaim)  │
│ Docker disk      43 GB    (29 GB recoverable) │
└─────────────────────────────────────────────┘
```

## Implementation notes

- `src/tui/app.rs` — new panels / modes
- Reuse memory/docker service layer from #46/#49
- `[R]` must not bypass `sw memory reclaim` safety checks

## References

- OrbStack memory requirements §12
- #46, #47, #49
