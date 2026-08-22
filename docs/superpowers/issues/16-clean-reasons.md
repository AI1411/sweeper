---
title: "[Post-MVP] Sharpen sw clean proposals (reasons + filters)"
labels: [enhancement, post-mvp]
depends_on: []
priority: medium
---

## Summary

Improve `sw clean` so candidates explain *why* they were proposed, and let users exclude noisy matches more easily.

## Motivation

Current heuristics (`dev` name + orphan/listen) work but feel opaque. Showing reasons builds trust; light filters reduce false positives.

## Acceptance criteria

- [ ] Each candidate shows reason flags, e.g. `listening`, `orphan-ppid`, `name:node`
- [ ] Output remains colorful and scannable (reuse `style` helpers)
- [ ] Optional exclude patterns (at least one of: flag, env, or simple config) — e.g. skip `python` or a PID
- [ ] Still never auto-kills; confirm flow unchanged in spirit
- [ ] Unit tests extend `tests/clean_propose.rs` for reason tagging

## Suggested UX

```text
Sweeper found possible leftovers:

✓ 2 candidate processes
  node    pid 1513  ports [3000]  reasons: listening, name:node
  vite    pid 1602  ports []      reasons: orphan-ppid, name:vite
```

## References

- Requirements §14 (`sw clean`), §23 Priority A/B adjacent
- `src/clean/mod.rs`, `src/commands/clean.rs`
