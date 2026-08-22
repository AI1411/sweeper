# Sweeper — GitHub Issues

Issue definitions under this directory. Create them with:

```bash
./scripts/create-github-issues.sh
```

Requires `gh` auth with permission to create issues (`repo` scope).

## MVP (done)

Plan: [`../plans/2026-08-21-sweeper-mvp.md`](../plans/2026-08-21-sweeper-mvp.md)

| # | File | Title |
|---|---|---|
| 1 | `01-scaffold.md` | Cargo scaffold + error types + ProcessInfo |
| 2 | `02-cli-target.md` | CLI target resolution |
| 3 | `03-protect.md` | Protected process list |
| 4 | `04-list-processes.md` | Process listing via sysinfo |
| 5 | `05-ports-lsof.md` | Port resolution via lsof |
| 6 | `06-kill-flow.md` | Kill flow (SIGTERM → wait → SIGKILL) |
| 7 | `07-history.md` | History store (JSON) |
| 8 | `08-cli-commands.md` | Name / port / top commands |
| 9 | `09-clean-history-cmd.md` | `sw clean` + `sw history` |
| 10 | `10-tui.md` | TUI (ratatui) |
| 11 | `11-ports-list-readme.md` | `sw ports` + README |

## Post-MVP

Suggested next features (high → medium).

| # | File | Title | Priority |
|---|---|---|---|
| 12 | `12-project.md` | `sw project` group/kill by project | high |
| 13 | `13-tree-kill.md` | Implement `--tree` process tree kill | high |
| 14 | `14-tui-ports.md` | Show listening ports clearly in TUI | high |
| 15 | `15-multi-port-ux.md` | Improve multi-port kill confirmation UX | medium |
| 16 | `16-clean-reasons.md` | Sharpen `sw clean` (reasons + filters) | medium |
| 17 | `17-memory-freed.md` | Report estimated memory freed after kill | medium |

## Quality (next)

| # | File | Title | Priority |
|---|---|---|---|
| 18 | `18-kill-summary-port-release.md` | Kill summary: ports released + richer feedback | high |
| 19 | `19-clean-reason-detail.md` | Human-readable `sw clean` reasons | high |
| 20 | `20-tui-navigation-kill-preview.md` | TUI navigation (g/G) + kill preview | high |

Suggested order: **18 → 19 → 20**.
