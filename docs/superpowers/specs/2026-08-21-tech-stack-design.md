# Sweeper 技術スタック設計

**日付:** 2026-08-21  
**ステータス:** Approved (design review)  
**関連:** [要件定義書](../../requirements.md)

## 1. 背景と決定

要件定義書 §21 では Go + Bubble Tea + Lip Gloss を想定していたが、言語を再選定した。

候補は Rust と Zig。Sweeper の核は TUI での選択・終了 UX であり、TUI/CLI エコシステムの成熟度から **Rust** を採用する。

周辺は「実用 MVP」方針とする。プロセス情報は当面 `sysinfo` と `lsof` のハイブリッドとし、純 OS API 化は後続で行う。

## 2. 全体構成

単一バイナリ `sw`。CLI と TUI は同じコアを共有する。

```text
sw (clap)
 ├── commands/     # top, ports, clean, history, project, name/port kill
 ├── tui/          # ratatui（一覧・検索・複数選択・確認）
 ├── process/      # 列挙・ツリー・ポート解決・kill
 ├── clean/        # leftover 候補の判定（提案のみ）
 └── history/      # JSON 履歴の読み書き
```

原則:

- **Sweeper proposes. User decides.** — `clean` も自動 kill しない
- デフォルトは SIGTERM。SIGKILL は `--force` / TUI の `K` のみ
- プロセス取得は当面ハイブリッド（sysinfo + lsof）

## 3. クレート構成と依存関係

単一クレート `sweeper`。バイナリ名は `sw`（`[[bin]] name = "sw"`）。

クレート分割はコード規模が大きくなってから検討する。

| 用途 | クレート | 備考 |
|---|---|---|
| 言語 | Rust (edition 2021 以上) | |
| CLI | `clap` (derive) | `sw <target>` とサブコマンドを両立 |
| TUI | `ratatui` + `crossterm` | 一覧・検索・選択・確認 |
| プロセス一覧 / CPU / MEM | `sysinfo` | |
| シグナル | `nix` | SIGTERM / SIGKILL |
| ポート | `std::process::Command` → `lsof` | MVP。後で native 置換可 |
| エラー | `thiserror` + `anyhow` | ライブラリ境界とバイナリ入口 |
| シリアライズ | `serde` + `serde_json` | history |
| パス | `directories` | macOS Application Support |
| 時刻 | `time` | history 表示（軽量優先） |

### MVP で採用しないもの

- async ランタイム（tokio 等）— 同期で足りる
- SQLite — history は JSON で十分
- 設定ファイルフレームワーク — 保護リストは後続
- 確認スキップ（`-y` / `--yes`）— 安全性優先で MVP 外

## 4. CLI / ターゲット解決

位置引数の解釈は一箇所に集約する。

```text
sw                  → TUI
sw node             → 名前検索（曖昧一致）
sw :3000            → ポート検索
sw :3000 :3001      → 複数ポート
sw ports | top | clean | history | project […]  → サブコマンド
```

解決ルール（優先順）:

1. 既知サブコマンド（`ports` / `top` / `clean` / `history` / `project`）
2. `:` 始まり → ポート（複数可）
3. それ以外 → プロセス名検索

共通オプション:

- `--force` … SIGKILL を許可（確認フローは維持）
- `--tree` … ツリー単位（Priority A。MVP ではスタブ可）

## 5. process 層と kill フロー

### データモデル

```text
ProcessInfo {
  pid, ppid, name, cpu, memory,
  ports[], command?, cwd?
}
```

`command` / `cwd` は後続（プロジェクト認識）で充実させる。MVP では取得できる範囲で埋める。

### 取得手段

| 情報 | MVP | 後続 |
|---|---|---|
| 一覧・CPU・MEM・名前 | `sysinfo` | 同左 |
| LISTEN ポート一覧 | `lsof -nP -iTCP -sTCP:LISTEN` | libproc 等 |
| 特定ポート | `lsof -nP -iTCP:PORT -sTCP:LISTEN` | 同左 |
| シグナル送信 | `nix::sys::signal::kill` | 同左 |

### kill フロー

要件 §15 に準拠する。

```text
対象確定 → 確認 [y/N]
  → SIGTERM
  → 短い待機（約 2 秒）
  → 生存確認
  → 残存なら Force? [y/N]
     （または --force 済みなら SIGKILL）
  → 結果表示 + history 追記
```

### 安全

- 自ユーザープロセスを優先表示
- 保護リスト（例: `launchd`, `WindowServer`, `kernel_task`）は kill 対象から除外または強い警告
- root 昇格は行わない

## 6. TUI / clean / history

### TUI（`sw`）

- `ratatui` による一覧 + `/` フィルタ + Space 複数選択
- `k` → SIGTERM 確認、`K` → SIGKILL 確認、`q` 終了
- 表示列: PID / PROCESS / PORT / CPU / MEM（要件 §6.2）
- ポート列は起動を阻害しないよう、別スレッド（`std::thread`）で `lsof` した結果をマージする。tokio 等は使わない

キーバインドの最終調整は実装時に行い、要件 §20 を基準とする。

### `sw clean`

候補提示のみ。ヒューリスティック例:

- 孤児っぽい開発系プロセス（node, vite, bun 等）
- LISTEN 中だが長時間アイドル寄り、など簡易ルール

自動 kill はしない。選択 → 確認 → 共通 kill フローへ。

### `sw history`

- 保存先: `~/Library/Application Support/sweeper/history.json`
- 記録項目: 日時, pid, name, ports, signal, 結果
- `sw history --last` で直前 1 件
- 上限 200 件。超過分は古いものから削除

## 7. 配布・テスト

### 配布

- 単一バイナリ
- Homebrew formula で配布（初期対応は macOS）

### テスト

- ユニットテスト: ターゲット解決、保護リスト、history の読み書き
- プロセス操作: モック可能な境界を `process` に置き、統合テストは必要最小限
- TUI: ロジックを view から分離し、モデル層を単体テスト可能にする

## 8. 要件定義書との差分

| 項目 | 旧（要件 §21） | 新 |
|---|---|---|
| 言語 | Go | **Rust** |
| TUI | Bubble Tea | **ratatui** + crossterm |
| スタイリング | Lip Gloss | ratatui / スタイル API |
| プロセス情報 | ps / lsof / kill | **sysinfo** + **lsof** + **nix** |

機能要件・MVP 範囲・優先度（要件 §22–§23）は変更しない。

## 9. この設計のスコープ外

実装計画・詳細 API・プロジェクト認識本実装・プロセスツリー本実装・Linux 対応は、本ドキュメントの範囲外とする。実装着手時は別途 implementation plan を作成する。
