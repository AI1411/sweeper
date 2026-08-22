---
title: "[UX] TUI process detail view (i / Enter)"
labels: [enhancement, post-mvp]
depends_on: []
priority: high
---

## Summary

Add a TUI detail panel for the current row showing PID, PPID, CPU, memory, ports, uptime, command, cwd, parent chain, and inferred project.

## Motivation

Requirements §12 describe `i` / Enter for process info. Kill preview is one line; full context before kill is core to Sweeper’s differentiation.

## Acceptance criteria

- [ ] `i` or `Enter` toggles detail view for cursor row (not multi-select summary only)
- [ ] Fields: PID, PPID, CPU, memory, ports, age/uptime, command, cwd, parent chain, project name
- [ ] Detail view does not block list navigation (toggle off with `i`, `Esc`, or `Enter`)
- [ ] Works with search filter and ports-only filter
- [ ] Footer updated when detail is open
- [ ] Unit tests for detail text formatting (pure function)

## Suggested UX

```text
node  PID 4812  PPID 4701  CPU 12.4%  MEM 421 MB  Port :3000  Started 2h 14m ago
Command: node ./node_modules/.bin/next dev
CWD: ~/dev/my-app
Parent: zsh → bun → node
Project: my-app
```

## Implementation notes

- `src/tui/ui.rs` — overlay or split pane
- `src/project/mod.rs` — `infer_project`
- `src/process/tree.rs` — parent chain helper if missing

## References

- Requirements §12 (プロセス詳細), §20 (TUI `i` key)
- `src/tui/{app,ui,mod}.rs`
