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

## Roadmap (quality & next)

Plan source: quality review (2026-08-22). Create with `./scripts/create-github-issues.sh roadmap`.

| # | File | Title | Priority |
|---|---|---|---|
| 21 | `21-tui-kill-confirm.md` | TUI kill confirmation | high |
| 22 | `22-protect-list-expand.md` | Expand protected list + user config | high |
| 23 | `23-clean-confidence.md` | Clean confidence hints / score sort | medium |
| 24 | `24-dry-run.md` | `--dry-run` mode | medium |
| 25 | `25-tui-process-detail.md` | TUI process detail (`i` / Enter) | high |
| 26 | `26-port-range.md` | Port range `sw :3000-3010` | medium |
| 27 | `27-top-interactive-kill.md` | Interactive kill from `sw top` | medium |
| 28 | `28-tui-project-view.md` | TUI project grouping view | medium |
| 29 | `29-tui-tree-visualization.md` | TUI process tree visualization | low |
| 30 | `30-macos-ci.md` | macOS GitHub Actions CI | high |
| 31 | `31-native-port-resolution.md` | Native port resolution (less lsof) | medium |
| 32 | `32-linux-formal-support.md` | Formal Linux support matrix | medium |
| 33 | `33-tui-smoke-tests.md` | TUI smoke tests | medium |
| 34 | `34-golden-cli-tests.md` | Golden CLI output tests | low |
| 35 | `35-ci-expansion.md` | CI expansion (doc, audit) | low |
| 36 | `36-mock-kill-integration-tests.md` | Mock kill integration tests | low |
| 37 | `37-github-releases.md` | GitHub Releases + binaries | medium |
| 38 | `38-json-output.md` | `--json` output mode | medium |
| 39 | `39-man-page-help.md` | Man page + enriched `--help` | low |
| 40 | `40-process-refresh-strategy.md` | Smarter TUI process refresh | low |
| 41 | `41-port-cache.md` | Port lookup cache (TTL) | low |
| 42 | `42-default-sort.md` | Developer-centric default sort | low |
| 43 | `43-framework-detection.md` | Expand framework detection | medium |
| 44 | `44-active-session-detection.md` | Active dev session vs leftovers | medium |
| 45 | `45-orphan-detection.md` | Cross-project orphan detection | medium |

Suggested order (phases):

1. **Safety:** 21, 22, 24
2. **UX gaps:** 25, 26, 27, 28
3. **Platform / CI:** 30, 31, 32, 35
4. **Testing:** 33, 34, 36
5. **Distribution / scripting:** 37, 38, 39
6. **Performance / polish:** 40, 41, 42
7. **Clean intelligence:** 23, 43, 44, 45
8. **Nice-to-have UX:** 29
