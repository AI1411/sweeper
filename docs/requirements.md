# Sweeper 要件定義書

## 1. プロジェクト概要

### 1.1 プロジェクト名

**Sweeper**

### 1.2 CLIコマンド名

```bash
sw
```

### 1.3 名前の由来

「Sweeper」は野球の変化球である**スイーパー**に由来する。

同時に英語の `sweep` には「掃く」「一掃する」という意味があり、不要なプロセスを見つけて整理・終了するツールのコンセプトとも一致する。

### 1.4 コンセプト

> **Sweep unwanted processes away.**

Sweeperは、macOS上で動作しているプロセスを確認・検索・選択し、素早く終了できる開発者向けCLI/TUIツール。

Activity Monitorやbtopのような「システム監視」を主目的とするのではなく、

**「開発中に不要になったプロセスを素早く見つけ、安全に片付ける」**

ことに特化する。

---

# 2. 解決したい課題

macOSではActivity Monitorからプロセスを確認・終了できるが、以下の問題がある。

- GUIを開く必要がある
- プロセスを1つずつ探す必要がある
- 複数プロセスをまとめて終了しにくい
- Node.jsなどが大量の子プロセスを生成する
- 開発サーバーがバックグラウンドに残ることがある
- 使用中のポートからプロセスを探すのが面倒
- PIDだけでは「どのプロジェクトのプロセスか」が分かりにくい
- `ps` / `lsof` / `kill` など複数のCLIコマンドを使い分ける必要がある
- `kill -9 $(lsof -ti:3000)` のようなコマンドは便利だが、安全性や対象の確認に乏しい

Sweeperではこれらを1つのCLI/TUIに統合する。

---

# 3. 基本思想

Sweeperは単なる「killコマンドのラッパー」にはしない。

以下を重視する。

1. **速い**
2. **短いコマンドで操作できる**
3. **対象が何なのか分かる**
4. **複数プロセスをまとめて扱える**
5. **開発プロジェクト単位で理解できる**
6. **誤って重要なプロセスを終了しにくい**
7. **通常終了を優先し、必要な場合のみ強制終了する**

---

# 4. 対象環境

## 初期対応

- macOS

## 将来的な対応候補

- Linux

Windowsについては初期スコープ外とする。

---

# 5. CLI基本設計

基本構文：

```bash
sw <target> [options]
```

`sw` 単体ではTUIを起動する。

```bash
sw
```

---

# 6. TUI

## 6.1 起動

```bash
sw
```

でインタラクティブなTUIを表示する。

例：

```text
┌ Sweeper ──────────────────────────────────────────┐
│ Search: _                                         │
├───────────────────────────────────────────────────┤
│   PID     PROCESS       PORT     CPU     MEM       │
│ > 4812    node          :3000     3%     420 MB    │
│   4921    bun           :8787     1%     180 MB    │
│   5102    postgres      :5432     2%     620 MB    │
│   6211    Cursor        -        12%     1.8 GB    │
│                                                   │
│ [Space] Select  [K] Kill  [/] Search  [Q] Quit    │
└───────────────────────────────────────────────────┘
```

## 6.2 表示情報

最低限以下を表示する。

- PID
- プロセス名
- CPU使用率
- メモリ使用量
- 使用ポート
- 選択状態

将来的には以下も表示可能にする。

- PPID
- 起動時間
- 実行コマンド
- Working Directory
- 所属プロジェクト

---

# 7. プロセス検索

## 7.1 名前検索

```bash
sw node
```

指定した文字列に関連するプロセスを検索する。

完全一致だけではなく、曖昧検索に対応する。

例：

```bash
sw cursor
```

```text
Found 7 processes

Cursor
├─ Cursor
├─ Cursor Helper
├─ Cursor Helper (GPU)
├─ Cursor Helper (Renderer)
└─ language_server

Total memory: 2.4 GB

Kill all? [y/N]
```

---

# 8. ポート検索・終了

Sweeperの主要機能の1つとする。

## 8.1 ポート指定

```bash
sw :3000
```

3000番ポートを使用しているプロセスを検索する。

例：

```text
PORT  PID    PROCESS     CPU    MEM
3000  48291  node        2.1%   184MB

Kill this process? [y/N]
```

## 8.2 複数ポート

```bash
sw :3000 :3001 :5173
```

複数ポートをまとめて検索・終了できる。

## 8.3 ポート範囲

将来的に以下にも対応する。

```bash
sw :3000-3010
```

指定範囲でLISTENしているプロセスを検索する。

## 8.4 ポート一覧

```bash
sw ports
```

現在LISTENしているポートをTUIで一覧表示する。

例：

```text
Sweeper / Ports

  PORT    PROCESS       PID      PROJECT
> 3000    node          48291    my-app
  3001    bun           49102    api-server
  5173    vite          51221    dashboard
  5432    postgres      2201     -
  8080    java          18211    backend

Space Select   K Kill   / Search   Q Quit
```

---

# 9. 複数選択・一括終了

TUI上で複数のプロセスを選択できる。

```text
Space
```

で選択状態を切り替える。

複数選択後、

```text
K
```

などのキーでまとめて終了する。

これによりActivity Monitorで1件ずつ終了する操作を不要にする。

---

# 10. プロセスツリー

親子関係を解析する。

例：

```text
node (PID 1200)
└─ next-server (PID 1204)
   ├─ worker (PID 1208)
   └─ worker (PID 1209)
```

プロセス単体だけでなく、プロセスツリー全体を終了できる。

CLI例：

```bash
sw node --tree
```

TUIでは、

```text
Kill process
Kill process tree
Cancel
```

のように選択できるようにする。

---

# 11. プロジェクト認識

Sweeperの差別化機能の1つ。

プロセスのWorking Directoryや実行コマンド、親子関係などから、

**「どの開発プロジェクトに属するプロセスか」**

を推定する。

例：

```bash
sw project
```

```text
my-app                    ~/dev/my-app

● bun dev
├─ vite           :5173
├─ hono           :3000
└─ worker

4 processes
812 MB
```

プロジェクト単位でまとめて終了できるようにする。

```bash
sw project my-app
```

---

# 12. プロセス詳細

TUI上でプロセスを選択して、

```text
i
```

などで詳細情報を表示する。

例：

```text
node

PID       4812
PPID      4701
CPU       12.4%
Memory    421 MB
Port      :3000
Started   2h 14m ago

Command
node ./node_modules/.bin/next dev

Working Directory
~/dev/my-app

Parent
zsh → bun → node

Project
my-app
```

---

# 13. `sw top`

CPU・メモリを多く消費しているプロセスを素早く確認する。

```bash
sw top
```

例：

```text
CPU

1. Cursor Helper       84%
2. node                42%
3. Chrome Helper       31%

MEMORY

1. Docker             4.2 GB
2. Cursor             2.8 GB
3. Chrome             1.9 GB
```

ここから対象を選択して終了できる。

---

# 14. `sw clean`

Sweeperを象徴するCleanup機能。

```bash
sw clean
```

Sweeperが不要である可能性のあるプロセスを分析する。

例：

```text
Sweeper found possible leftovers:

✓ 3 stale dev servers
✓ 2 orphan processes
✓ 1 zombie process
✓ 4 unused listening ports

Estimated memory reclaim: 1.4 GB

Select processes to clean →
```

## 重要

Sweeperが自動的に危険なプロセスを終了しないこと。

基本思想：

> **Sweeper proposes. User decides.**

Sweeperは候補を提示し、ユーザーが最終的に選択する。

---

# 15. Kill方式

## 15.1 通常終了

デフォルトではSIGTERMを使用する。

```bash
sw :3000
```

基本フロー：

```text
SIGTERM
   ↓
一定時間待機
   ↓
プロセス存在確認
   ↓
終了していない
   ↓
Force kill? [y/N]
   ↓
SIGKILL
```

## 15.2 強制終了

明示的なオプションを指定した場合のみSIGKILLを利用できる。

```bash
sw :3000 --force
```

---

# 16. 安全機能

システム上重要なプロセスを誤って終了しないようにする。

初期状態では、

- 自分のユーザーが起動したプロセスを優先
- OSの重要プロセスを非表示または保護
- root権限が必要な操作を安易に実行しない
- SIGTERMをデフォルトにする
- SIGKILLは明示的な操作に限定する
- 一括終了時は対象を確認できるようにする

重要プロセスについてはブラックリスト/保護リストを設けることも検討する。

---

# 17. Kill履歴

Sweeperから終了したプロセスの履歴を保存する。

```bash
sw history
```

例：

```text
10:42  node       PID 4812    :3000
10:31  vite       PID 4211    :5173
09:58  Cursor     8 processes
```

直前の操作：

```bash
sw history --last
```

履歴には必要に応じて以下を保存する。

- 日時
- PID
- プロセス名
- ポート
- プロジェクト
- Signal
- 終了結果

---

# 18. Cleanup結果

終了後に可能であれば、

```text
Killed 7 processes.

Memory reclaimed: ~1.8 GB
Ports released:
  :3000
  :3001
  :5173
```

のような結果を表示する。

これにより「何を片付けたのか」を明確にする。

---

# 19. CLIコマンド一覧

想定する主要コマンド：

```bash
# TUI起動
sw

# プロセス名から検索
sw node

# ポートから検索
sw :3000

# 複数ポート
sw :3000 :3001 :5173

# ポート一覧
sw ports

# プロセスツリー
sw node --tree

# プロジェクト一覧
sw project

# プロジェクト指定
sw project my-app

# CPU/メモリ上位
sw top

# Cleanup候補
sw clean

# 履歴
sw history

# 直前の履歴
sw history --last

# 強制終了
sw :3000 --force
```

---

# 20. TUIキーバインド案

| Key | Action |
|---|---|
| `↑ / ↓` | カーソル移動 |
| `Space` | 選択/解除 |
| `/` | 検索 |
| `Enter` | 詳細表示 |
| `i` | プロセス情報 |
| `k` | SIGTERM |
| `K` | SIGKILL |
| `t` | プロセスツリー |
| `p` | ポート表示 |
| `g` | プロジェクト/グループ表示 |
| `q` | 終了 |

キーバインドは実装時に最終調整する。

---

# 21. 技術スタック

詳細は [技術スタック設計](./superpowers/specs/2026-08-21-tech-stack-design.md) を正とする。

## 言語

**Rust**

理由：

- CLI / TUI エコシステムが成熟している（clap, ratatui）
- 単一バイナリとして配布できる
- プロセス操作・シグナル制御のクレートが揃っている
- Homebrew で配布しやすい
- 類似ツール（例: bottom）の先行事例がある

（旧案の Go は採用しない。）

## CLI

**clap**（derive）

## TUI

**ratatui** + **crossterm**

## プロセス情報

- 一覧 / CPU / MEM: **sysinfo**
- シグナル: **nix**（SIGTERM / SIGKILL）
- ポート: 当面 **lsof** 呼び出しを許容

ポート検索例：

```bash
lsof -nP -iTCP:3000 -sTCP:LISTEN
```

将来的には macOS API（libproc 等）による直接取得も検討する。

## その他（MVP）

- 履歴: JSON（`~/Library/Application Support/sweeper/`）
- エラー: thiserror + anyhow
- 配布: 単一バイナリ + Homebrew

---

# 22. MVP

最初から全機能を実装しない。

## MVP必須機能

### 1. TUI

```bash
sw
```

- プロセス一覧
- CPU
- Memory
- PID
- 検索
- 複数選択
- Kill

### 2. プロセス名検索

```bash
sw node
```

### 3. ポート検索

```bash
sw :3000
```

### 4. Top

```bash
sw top
```

### 5. Clean

```bash
sw clean
```

---

# 23. MVP後の優先機能

優先順位：

### Priority A

- プロセスツリー
- LISTENポート一覧
- 複数ポート終了
- Graceful Shutdown
- システムプロセス保護

### Priority B

- プロジェクト認識
- Working Directory取得
- コマンドライン解析
- プロジェクト単位Kill

### Priority C

- Kill履歴
- Cleanup結果
- メモリ解放量推定
- Linux対応

---

# 24. Sweeperの差別化

既存ツールとの方向性を明確に分ける。

### Activity Monitor

GUIによるシステム監視・プロセス管理。

### btop / bottom

ターミナル上でのシステムリソース監視。

### ps / lsof / kill

Unix標準の低レベルなプロセス操作。

### Sweeper

**開発中に残った不要なプロセスを、開発者目線で理解・選択・終了するためのツール。**

特に、

- `sw :3000`
- 複数選択Kill
- プロセスグルーピング
- プロセスツリー
- プロジェクト認識
- `sw clean`

を中心的な差別化要素とする。

---

# 25. UX原則

Sweeperではコマンド体系を可能な限り、

```bash
sw <target>
```

に統一する。

例：

```bash
sw
sw node
sw cursor
sw :3000
sw :5173
sw project
sw top
sw clean
```

ユーザーがPIDを調べてから操作するのではなく、

> **「何を止めたいか」を指定する**

だけで操作できることを目標とする。

---

# 26. 将来的な方向性

Sweeperは単なるプロセスモニターではなく、

**Developer Process Cleanup Tool**

として発展させる。

将来的には、

- Node.js
- Bun
- Vite
- Next.js
- Hono
- Python
- Docker
- Playwright
- language server

などの開発系プロセスを認識し、

```text
Process
↓
Process Tree
↓
Port
↓
Working Directory
↓
Project
```

を関連付ける。

最終的には、

**「PIDを管理する」のではなく「開発環境を理解して片付ける」**

ツールを目指す。