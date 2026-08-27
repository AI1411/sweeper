---
title: "[Project] Git branch, docker-compose, and dev-script detection"
labels: [enhancement, post-mvp]
depends_on: []
priority: medium
---

## Summary

Extend `sw project` and TUI project view with git branch labels, docker-compose grouping, and package.json dev-script inference.

## Motivation

Developers often run multiple branches or compose stacks. Knowing **which branch** or **which compose project** a process belongs to makes kill decisions safer and faster.

## Acceptance criteria

- [ ] Show git branch (best-effort via `git -C <cwd> branch --show-current`) in project summary and TUI
- [ ] Detect `docker-compose.yml` / `compose.yaml` at project root and group related processes
- [ ] Infer dev script name from command line (e.g. `pnpm dev`, `npm run start`) and display in project tree
- [ ] `--json` output includes `git_branch`, `compose_project`, `dev_script` when available
- [ ] Graceful fallback when git/compose metadata unavailable (no error)
- [ ] Tests with fixture cwd/command strings

## Implementation notes

- `src/project/mod.rs` — extend `ProjectGroup` / `ProjectMetadata`
- `src/tui/ui.rs` — show branch in project table
- Avoid spawning git on every refresh; cache per project path with TTL

## References

- `src/project/mod.rs`, `src/commands/project.rs`
- Related: #130 (tmux/monorepo workspace recognition)
