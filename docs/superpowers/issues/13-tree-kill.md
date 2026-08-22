---
title: "[Post-MVP] Implement --tree process tree kill"
labels: [enhancement, post-mvp]
depends_on: []
priority: high
---

## Summary

Make `--tree` actually kill a process and its descendants. Today the flag is accepted but tree kill is a stub.

## Motivation

Node / Vite / Next often leave child processes. Name search and clean are much more useful if the whole tree can be terminated safely.

## Acceptance criteria

- [ ] `sw <name> --tree` and `sw :<port> --tree` kill the matched root(s) and descendants
- [ ] TUI offers “Kill process” vs “Kill process tree” (or equivalent) for selected PIDs
- [ ] Descendants are discovered via PPID relationships from the process snapshot
- [ ] Still SIGTERM-first; SIGKILL only with `--force` / TUI `K`
- [ ] Protected names in the tree are skipped (and reported)
- [ ] History records which PIDs were targeted
- [ ] Tests cover tree collection logic with fixture parent/child graphs

## Suggested UX

```bash
sw node --tree
sw :3000 --tree --force
```

TUI:

```text
Kill process
Kill process tree
Cancel
```

## References

- Requirements §10 (process tree), §23 Priority A
- `src/cli.rs` (`--tree` stub note)
- `src/process/kill.rs`
