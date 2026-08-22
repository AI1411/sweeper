---
title: "[Quality] Clean candidate confidence hints and score-based sort"
labels: [enhancement, post-mvp]
depends_on: []
priority: medium
---

## Summary

Show confidence hints (`high` / `medium`) for `sw clean` candidates, or sort by internal score so the most likely leftovers appear first.

## Motivation

Human-readable reasons (#49) improved trust, but users still scan a flat list. Confidence or score ordering helps focus on safe-to-kill candidates first without auto-killing.

## Acceptance criteria

- [ ] Each candidate has a confidence level derived from existing `CleanCandidate` scoring in `src/clean/mod.rs`
- [ ] CLI output shows hint (e.g. `confidence: high`) or sorts by score (document choice)
- [ ] High-confidence examples: zombie, orphan with missing parent, stale listener 4h+
- [ ] Medium-confidence examples: idle listener, stack hint only
- [ ] Confirm flow unchanged; no auto-kill
- [ ] Unit tests for confidence mapping from reason tags / score

## Suggested UX

```text
  node  pid 1513  ports [3000]  age 5h  confidence: high
    reasons: stack:vite, stale-server (5h on :3000)
```

## Implementation notes

- `format_reasons_display` / `summarize` in `src/clean/mod.rs`
- `src/commands/clean.rs` for output

## References

- Requirements §14 (`sw clean`), issue #49 (human-readable reasons)
- `src/clean/mod.rs`, `tests/clean_propose.rs`
