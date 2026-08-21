# Sweeper MVP — GitHub Issues

実装計画 [`../plans/2026-08-21-sweeper-mvp.md`](../plans/2026-08-21-sweeper-mvp.md) の Task 1–11 を Issue 化した定義です。

この環境の GitHub トークンは **Issue 作成権限がない**ため、ローカルで次を実行してください。

```bash
./scripts/create-github-issues.sh
```

前提: `gh` 認証済み、`repo` スコープで Issue 作成可能であること。

| # | ファイル | タイトル |
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

依存関係:

```text
1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 10 → 11
         └───────────────┘
```

Task 3 は Task 4 と並列可。Task 5 は Task 4 の後。Task 6 は Task 3 必須。
