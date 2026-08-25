---
title: "[Project] tmux session and monorepo workspace recognition"
labels: [enhancement, post-mvp]
depends_on: []
priority: medium
---

## Summary

Improve Sweeper's developer-centric grouping by recognizing **tmux/screen sessions** and **monorepo workspaces** (pnpm, npm workspaces, Turbo, Nx) so `sw project` and TUI project view reflect how developers actually organize work.

## Motivation

Today `sw project` infers groups mainly from working directory and command line. In real dev environments:

- Multiple apps in one repo (monorepo) appear as a single project or split incorrectly
- Processes started in tmux panes belong to a **session context** that is more meaningful than raw cwd alone
- `git worktree` checkouts share a repo name but are separate working contexts

Better grouping strengthens Sweeper's differentiation: "understand the dev environment, not just PIDs."

## Acceptance criteria

### Monorepo / workspace detection

- [ ] Detect workspace root markers: `pnpm-workspace.yaml`, root `package.json` with `workspaces`, `turbo.json`, `nx.json`
- [ ] Group processes under **workspace package name** when cwd is inside a package (e.g. `apps/web` → project `my-monorepo/web`)
- [ ] `sw project --json` includes optional fields: `workspace_root`, `package_name` (when detected)
- [ ] `git worktree` aware: different worktree paths → distinct project groups even if folder basename matches
- [ ] Unit tests with fixture directory layouts (no live git required in CI — use path fixtures)

### tmux / screen session grouping

- [ ] Detect when parent chain includes `tmux` or `screen`
- [ ] Attach session label to `ProcessInfo` or project metadata (e.g. `tmux:my-dev`)
- [ ] `sw project` output shows session name when present
- [ ] TUI project view (`P`): optional column or subtitle for tmux session
- [ ] Graceful no-op when tmux/screen not running or session name unavailable

### UX / safety

- [ ] Kill and protect behavior unchanged; grouping is display + targeting only
- [ ] Document limitations (remote tmux, nested sessions) in README

## Suggested output

```text
$ sw project
my-monorepo/web     ~/dev/my-monorepo/apps/web   tmux:api-dev
  ● vite            :5173
  ● node            :3000
  2 processes  420 MB

my-monorepo/api     ~/dev/my-monorepo/apps/api   tmux:api-dev
  ● bun             :8787
  1 process   180 MB
```

## Implementation notes

- `src/project/mod.rs` — `infer_project`, new `infer_workspace_package`, `infer_tmux_session`
- Walk up from cwd for workspace markers (cap depth, e.g. 8 levels)
- tmux: parse `TMUX` env from `/proc/<pid>/environ` on Linux; macOS equivalent via sysinfo/cmdline if available
- `src/process/types.rs` — optional `session_label: Option<String>`
- `tests/` — project grouping fixtures

## References

- Requirements §11 (プロジェクト認識), §26
- Closed: #62 (TUI project view) — extend grouping data fed to TUI
- `src/project/mod.rs`, `src/commands/project.rs`

## Out of scope (follow-ups)

- Kill by tmux session name (`sw tmux api-dev`) — separate issue if needed
- Docker Compose service name mapping — see OrbStack/Docker issues
