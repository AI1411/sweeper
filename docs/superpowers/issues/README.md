# Sweeper MVP — GitHub Issues

実装計画 [`../plans/2026-08-21-sweeper-mvp.md`](../plans/2026-08-21-sweeper-mvp.md) の Task 1–11 を Issue 化した定義です。

再作成が必要な場合:

```bash
./scripts/create-github-issues.sh
```

前提: `gh` 認証済み、`repo` スコープで Issue 作成可能であること。

| # | GitHub | ファイル | タイトル |
|---|---|---|---|
| 1 | [#5](https://github.com/AI1411/sweeper/issues/5) | `01-scaffold.md` | Cargo scaffold + error types + ProcessInfo |
| 2 | [#6](https://github.com/AI1411/sweeper/issues/6) | `02-cli-target.md` | CLI target resolution |
| 3 | [#7](https://github.com/AI1411/sweeper/issues/7) | `03-protect.md` | Protected process list |
| 4 | [#8](https://github.com/AI1411/sweeper/issues/8) | `04-list-processes.md` | Process listing via sysinfo |
| 5 | [#9](https://github.com/AI1411/sweeper/issues/9) | `05-ports-lsof.md` | Port resolution via lsof |
| 6 | [#10](https://github.com/AI1411/sweeper/issues/10) | `06-kill-flow.md` | Kill flow (SIGTERM → wait → SIGKILL) |
| 7 | [#11](https://github.com/AI1411/sweeper/issues/11) | `07-history.md` | History store (JSON) |
| 8 | [#12](https://github.com/AI1411/sweeper/issues/12) | `08-cli-commands.md` | Name / port / top commands |
| 9 | [#13](https://github.com/AI1411/sweeper/issues/13) | `09-clean-history-cmd.md` | `sw clean` + `sw history` |
| 10 | [#14](https://github.com/AI1411/sweeper/issues/14) | `10-tui.md` | TUI (ratatui) |
| 11 | [#15](https://github.com/AI1411/sweeper/issues/15) | `11-ports-list-readme.md` | `sw ports` + README |

依存関係:

```text
1 (#5) → 2 (#6) → 3 (#7) → 4 (#8) → 5 (#9) → 6 (#10) → 7 (#11) → 8 (#12) → 9 (#13) → 10 (#14) → 11 (#15)
              └───────────────┘
```

Task 3 は Task 4 と並列可。Task 5 は Task 4 の後。Task 6 は Task 3 必須。
