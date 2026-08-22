---
title: "[Clean] Expand framework and stack detection"
labels: [enhancement, post-mvp]
depends_on: []
priority: medium
---

## Summary

Tune `NAME_HINTS` and `CMD_PATTERNS` in `sw clean` for Docker, Playwright, language servers, and common 2026 dev stacks; reduce false positives.

## Motivation

Requirements §26 list Node, Bun, Vite, Docker, Playwright, language servers. Detection is partial; real-world leftovers are missed or mis-tagged.

## Acceptance criteria

- [ ] Add patterns for: Docker Desktop proxy, Playwright browsers, `uvicorn`/`fastapi`, `pnpm`, `astro`, ESLint server
- [ ] Document each pattern in code comment with example command line
- [ ] False positive review: do not flag active `postgres` / `redis` without idle/stale signals
- [ ] Tests in `tests/clean_propose.rs` for new patterns
- [ ] Optional: `stack:` reason shows detected framework name in display (#49 format)

## Implementation notes

- `src/clean/mod.rs` — `NAME_HINTS`, `CMD_PATTERNS`
- Gather examples from issue description or fixture strings

## References

- Requirements §26 (将来的な方向性)
- `src/clean/mod.rs`, `tests/clean_propose.rs`
