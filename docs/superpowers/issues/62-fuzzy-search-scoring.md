---
title: "[UX] Fuzzy search scoring for name and command matching"
labels: [enhancement, post-mvp]
depends_on: []
priority: medium
---

## Summary

Replace plain substring matching with scored fuzzy search for `sw <name>` and TUI `/` search.

## Motivation

`find_by_name_fuzzy` is substring-only (`name.contains(query)`). Queries like `nxt` won't match `next-server`, and results aren't ranked by relevance.

## Acceptance criteria

- [ ] Score matches: exact name > prefix > substring > command-line match
- [ ] CLI `sw node` shows results sorted by score (best first)
- [ ] TUI search filters in real time with same scoring
- [ ] Optional: highlight matched substring in TUI (stretch)
- [ ] Tests for scoring edge cases (empty query, case insensitivity)
- [ ] No new heavy dependencies unless justified (simple scorer OK)

## Implementation notes

- `src/process/list.rs` — `score_name_match(query, name, command) -> u32`
- `src/tui/app.rs` — apply score in `refilter`

## References

- `src/process/list.rs` (`find_by_name_fuzzy`, `name_matches`)
