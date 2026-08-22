---
title: "[Clean] Distinguish active dev session from leftovers"
labels: [enhancement, post-mvp]
depends_on: [43-framework-detection]
priority: medium
---

## Summary

Reduce false positives in `sw clean` by detecting “likely active” dev sessions (recent CPU, TTY, parent shell, IDE-launched) and down-ranking or excluding them.

## Motivation

A vite server started 10 minutes ago from the IDE should not appear next to a 5-hour stale listener. Active-session heuristics build trust in clean proposals.

## Acceptance criteria

- [ ] Heuristics documented: e.g. run_time < 15m + CPU > threshold → not stale-server candidate
- [ ] Parent is interactive shell (`zsh`, `bash`, `fish`) + young process → lower confidence or exclude from orphan rules
- [ ] Still never auto-kill; only affects proposal set or confidence (#23)
- [ ] Unit tests with fixture processes for active vs stale cases
- [ ] No regression: true stale servers (4h+, low CPU) still proposed

## Implementation notes

- `src/clean/mod.rs` — `is_likely_active_session(proc: &ProcessInfo)`
- May interact with confidence scoring (#23)

## References

- Requirements §14 (`sw clean`), §26
- `src/clean/mod.rs`
