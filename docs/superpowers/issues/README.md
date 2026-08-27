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

## OrbStack / Developer Resource Manager (2026-08)

OrbStack memory & Docker resource management. Create with:

```bash
for f in docs/superpowers/issues/{46..55}-*.md; do
  # see scripts/create-github-issues.sh or create manually via gh
done
```

| # | File | GitHub | Title | Priority |
|---|---|---|---|---|
| 46 | `46-sw-memory.md` | #107 | `sw memory` — OrbStack/Docker memory analysis | high |
| 47 | `47-sw-memory-reclaim.md` | #108 | `sw memory reclaim` — safe reclaim | high |
| 48 | `48-sw-memory-warnings.md` | #109 | High memory usage warnings | medium |
| 49 | `49-sw-docker.md` | #110 | `sw docker` — resource overview | medium |
| 50 | `50-sw-disk.md` | #111 | `sw disk` — Docker/dev storage | medium |
| 51 | `51-tui-orbstack.md` | #112 | TUI OrbStack/Docker dashboard | medium |
| 52 | `52-sw-memory-watch.md` | #113 | `sw memory watch` | low |
| 53 | `53-memory-leak-detection.md` | #114 | Container memory leak detection | low |
| 54 | `54-sw-clean-orbstack.md` | #115 | Integrate reclaim into `sw clean` | low |
| 55 | `55-sw-cache.md` | #116 | `sw cache` — dev tool caches | low |

Suggested order:

1. **Foundation:** 46 → 47, 50
2. **Overview:** 49 (needs 46 + 50)
3. **UX:** 48, 51
4. **Monitoring:** 52 → 53
5. **Integration:** 54, 55

## Improvement backlog (2026-08 review)

Created from codebase review. Files `56–70`; GitHub #137–#151.

| File | GitHub | Title | Priority |
|---|---|---|---|
| 56 | #137 | `[Clean]` CLI batch confirm for `sw clean` | high |
| 57 | #138 | `[Project]` Git branch, docker-compose, dev-script detection | medium |
| 58 | #139 | `[History]` Project metadata, filters, quick re-watch | low |
| 59 | #140 | `[UX]` TUI context-sensitive help footer | high |
| 60 | #141 | `[UX]` TUI auto-refresh CPU/MEM on interval | high |
| 61 | #142 | `[UX]` TUI sort toggle | medium |
| 62 | #143 | `[UX]` Fuzzy search scoring | medium |
| 63 | #144 | `[Config]` User config file (`config.toml`) | medium |
| 64 | #145 | `[Feature]` Extend `--json` to `sw top` | low |
| 65 | #146 | `[Performance]` Reuse sysinfo `System` for TUI refresh | high |
| 66 | #147 | `[Performance]` Batch port lookup via LISTEN cache | medium |
| 67 | #148 | `[Performance]` HashMap-based `merge_ports` | medium |
| 68 | #149 | `[Performance]` TUI conditional redraw | medium |
| 69 | #150 | `[Performance]` Parallel SIGTERM wait for batch kills | low |
| 70 | #151 | `[Performance]` Faster `sw cache` dir size scan | low |

Suggested order:

1. **High impact UX:** 59, 60, 56
2. **Performance:** 65, 66, 67, 68
3. **Polish:** 61, 62, 63, 57, 58, 64, 69, 70

Already covered by earlier closed issues (no new issue needed):

- Orphan detection → #79 (implemented)
- Memory leak detection → #114 (implemented)
- Default sort → #76 (implemented)
- Port cache TTL → #75 (implemented)
- Multi-port batch confirm → #15 (implemented in `src/commands/port.rs`)
