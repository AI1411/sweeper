---
title: "[Clean] Strengthen active-session detection and confidence UX"
labels: [enhancement, post-mvp]
depends_on: []
priority: high
---

## Summary

Build on the initial active-session heuristics (#78) and confidence scoring (#57) to further reduce `sw clean` false positives and make confidence levels obvious in both CLI and TUI.

## Motivation

Basic active-session detection and CLI `confidence:` hints exist, but users still hesitate to use `sw clean` daily when:

- Active dev servers appear next to true leftovers
- Confidence is easy to miss in plain text output
- TUI does not surface confidence or active-session status
- Heuristics do not cover common cases (TTY-attached, IDE-launched, recently restarted)

Stronger detection plus clearer confidence UX directly reduces fear of accidental kills.

## Acceptance criteria

### Active-session detection (extend `is_likely_active_session`)

- [ ] Document all heuristics in `src/clean/mod.rs` with examples
- [ ] TTY-attached listener with interactive parent shell → treat as active session
- [ ] Parent chain includes known IDE launcher (`Cursor`, `Code Helper`, `idea`, etc.) + young process → active session
- [ ] Process restarted within last N minutes (same cwd + command hash in history) → exclude from stale-server for one cycle (optional, document if deferred)
- [ ] No regression: 4h+ low-CPU listeners still proposed; zombies/orphans still high confidence

### Confidence UX

- [ ] CLI: confidence shown prominently per candidate (`high` / `medium` / `low` if introduced)
- [ ] CLI: default sort by confidence (high first), then score; document in README
- [ ] TUI (`sw clean` flow or clean-oriented view): confidence badge per row with color (reuse `style.rs`)
- [ ] TUI: optional filter key or flag to show only `high` confidence candidates
- [ ] `--json`: stable `confidence` field (already present — verify schema + tests)

### Safety

- [ ] Still never auto-kill; confirmation flow unchanged
- [ ] Protected processes unchanged

### Tests

- [ ] Unit tests for new active-session fixtures (TTY, IDE parent, edge cases)
- [ ] Golden or snapshot test for sorted confidence output

## Suggested UX

```text
$ sw clean
Sweeper found possible leftovers (sorted by confidence):

  HIGH   node  pid 1513  :3000  age 5h  stack:vite
         reasons: stale-server (5h on :3000)

  MEDIUM bun   pid 2201  :8787  age 45m
         reasons: idle-listener (low CPU, listening)
```

TUI row example: `[HIGH] node 1513  :3000  5h  vite`

## Implementation notes

- `src/clean/mod.rs` — `is_likely_active_session`, `confidence_level`, sort helper
- `src/commands/clean.rs` — CLI output + sort
- `src/tui/` — badge rendering if clean is exposed in TUI (or shared candidate list widget)
- `tests/clean_propose.rs` — new fixtures

## References

- Closed: #57 (confidence hints), #78 (active session)
- Requirements §14 (`sw clean`), §26
- `src/clean/mod.rs`, `src/commands/clean.rs`
