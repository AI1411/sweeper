# Sweeper MVP — GitHub Issues

実装計画 [`../plans/2026-08-21-sweeper-mvp.md`](../plans/2026-08-21-sweeper-mvp.md) の Task 1–11 を Issue 化した定義です。

Cloud Agent 組み込みの GitHub App トークンは **Issue 作成権限がない**ことがあります。  
Personal Access Token を **`GITHUB_PAT`** として登録し、次を実行してください。

```bash
./scripts/create-github-issues.sh
```

### Cursor Cloud にトークンを登録する

1. GitHub で Fine-grained PAT を作成（この repo に Issues: Read and write）
2. [Cloud Agents → Secrets](https://cursor.com/dashboard/cloud-agents) を開く
3. Secret を追加:
   - Name: `GITHUB_PAT`（`GH_TOKEN` は使わない — Cursor 側と衝突しやすい）
   - Type: **Runtime Secret**
   - Value: 作成した PAT
4. **新しい** Cloud Agent を起動（既存ランには注入されないことがある）

ローカルなら:

```bash
export GITHUB_PAT=ghp_...   # チャットに貼らない
./scripts/create-github-issues.sh
```

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
