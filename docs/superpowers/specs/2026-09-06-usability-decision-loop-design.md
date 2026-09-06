# Usability: Decision Loop

**日付:** 2026-09-06  
**ステータス:** Phase 1 implemented  
**関連:** [要件定義書](../../requirements.md) §25 UX原則

## 1. 問題

Sweeper の機能面（TUI、`clean`、プロジェクト認識、保護リスト）は揃っている。残っている摩擦は機能不足ではなく、**判断に必要な情報が足りない**ことと、**判断の単位が粗い**ことである。

今日の `sw node`:

- ポート・プロジェクト・cwd が出ない（`find_by_name_fuzzy` はポートをマージしない）
- 複数ヒット時の操作は `Kill all? [y/N]` のみ
- `n` のあとに個別選択できない（`sw clean` だけ番号選択がある）
- 番号選択は `1,3` のみ。範囲・`all`・`high`・空 Enter キャンセルが無い

今日の TUI:

- 検索は必ず `/` が先。文字を打っても何も起きない
- フッターが `[[↑↓]] Move` と二重ブラケットになる
- キーが多く、Discoverability は `?` 依存

要件 §25 の目標は「PID を調べてから操作するのではなく、止めたいものを指定する」こと。指定したあと **どれがそれか分かる** ところまでが、同じ原則の続きである。

## 2. 方針の候補

### A. Undo / Revive（却下・将来）

履歴に command + cwd を足し、`sw undo` でプロセスを再起動する。誤 kill の恐怖は下がる。

- 長所: 安全性の印象が強い
- 短所: env / 対話入力 / デーモン化を復元できない。偽の Undo は信頼を壊す

Phase 2 候補として残す。本物の再起動ではなく「最後に殺したポートを `sw watch` する」程度なら安全。

### B. TUI コマンドパレット（却下・将来）

`:` で `clean` / `ports` / `protect` を選ぶ。キー過多の根本治療。

- 長所: 学習コストを一箇所に集約できる
- 短所: CLI の日常導線（`sw :3000`, `sw node`）には効かない。TUI だけの投資になる

### C. Decision Loop — Inspect → Select → Confirm（採用）

全 CLI 殺傷フローを同じ判断ループにする。

1. **Inspect** — 番号付きで、ポート・プロジェクト・PID・名前を出す
2. **Select** — 共通の選択文法（`all` / `1-3` / `high` / `q`）
3. **Confirm** — 既存の `[y/N]`。`-y` は追加しない

TUI 側は同じ「見つけやすくする」軸で、未バインド文字の type-to-filter とフッター修正を入れる。

- 長所: 既存思想（Sweeper proposes. User decides.）に沿う。CLI と TUI の両方に効く。自動 kill を増やさない
- 短所: Undo は無い。誤 kill 後の回復はこれまで通り履歴を見るだけ

## 3. 採用設計

### 3.1 共通選択文法

入力は trim し、カンマと空白を同じ区切りとみなす。

| 入力 | 意味 |
| --- | --- |
| `q` / `quit` / `n` / `no` / 空 | キャンセル |
| `all` / `a` / `*` | 全件 |
| `high` / `h` | high confidence のみ（渡されたときだけ有効） |
| `1,3` / `1 3` | 1-based インデックス |
| `1-3` | 閉区間 |
| `1-3,5` | 混在。重複は除去して昇順 |

範囲の逆転・0・範囲外は無効 → 既存どおり "Invalid selection." して殺さない。

`sw clean` / `sw node` / `sw :port` が同じパーサを使う。

### 3.2 Inspect 行

名前検索・ポート検索の一覧は番号付き 1 行:

```text
#    PID     PROCESS       PORT          PROJECT         CPU     MEM
  1.  4812  node          :3000         my-app          2.1%    184 MB
```

- `sw node` はヒットに LISTEN ポートをマージしてから表示する
- プロジェクトは既存 `infer_project`
- ポート無し・プロジェクト無しは `-`

複数件は `Kill all N processes? [y/N]`。No のあと `Kill which?` で選択文法。1 件は `Kill this process?`。

### 3.3 TUI

- 未バインドの印字文字は検索を開始し、その文字をクエリに入れる（`/n` 相当を `n` 一打で）
- 検索は PID 部分一致も見る（`4812` でその行に跳べる）
- フッターは `[↑↓] Move` 形式。キー側に括弧を二重に付けない
- バインド済みキー（`k` / `j` / `p` など）は変えない。vim の `k` = kill は維持

### 3.4 やらないこと（この PR）

- `-y` / 確認スキップ
- プロセスの再起動 Undo
- `sw protect` CLI（ファイル編集のまま。Phase 2）
- コマンドパレット
- 名前検索の複数クエリ（`sw node python` は第一引数のみ。既存）

## 4. データフロー

```text
sw node / sw :3000 / sw clean
        │
        ▼
   番号付き Inspect 行（port + project）
        │
        ▼
   Kill all? [y/N]
        │ no
        ▼
   parse_selection  ── all / high / 1-3 / q
        │
        ▼
   既存 kill_pid + history + summary
```

## 5. エラー扱い

- 無効入力: 殺さない。警告して終了（clean の現行）
- `high` を high リスト無しで打つ: 無効
- 保護プロセス: 既存 `SkippedProtected`
- `--dry-run` / `--json`: 選択プロンプトに入らない（現行）

## 6. テスト

- 選択パーサ: 範囲、混在、all、high、空キャンセル、無効
- Inspect 行: ポートとプロジェクトが出る（NO_COLOR）
- TUI: 未バインド文字で `searching` と query が入る。PID クエリでヒット
- 既存 golden CLI は clean ブロック形式を変えない

## 7. Phase 2（提案のみ）

1. `sw protect add <name>` / `sw protect list` — 誤って殺しそうになった名前をその場で守る
2. 履歴に command / cwd を残し、`sw history --last` から「同じポートを watch」まで誘導
3. TUI `:` コマンドパレット（clean / projects / protect）
4. 複数名 `sw node python` を AND/OR ではなく「複数ターゲット」として扱う
