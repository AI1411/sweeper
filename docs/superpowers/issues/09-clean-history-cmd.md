---
title: "[MVP] Task 9: sw clean + sw history"
labels: [enhancement, mvp]
depends_on: [8]
plan: docs/superpowers/plans/2026-08-21-sweeper-mvp.md#task-9
---

## Summary

`sw clean` は候補提示のみ（自動 kill 禁止）。`sw history` / `--last` を実装。

## Acceptance criteria

- [ ] `propose_leftovers` が開発系 + orphan/LISTEN のヒューリスティック
- [ ] clean は一覧 → 個別確認 → kill（自動終了なし）
- [ ] `sw history` が全件、`sw history --last` が直前 1 件
- [ ] 原則 **Sweeper proposes. User decides.** を守る

## Files

- Create: `src/clean/mod.rs`, `src/commands/clean.rs`, `src/commands/history.rs`
- Modify: `src/main.rs`, `src/lib.rs`, `src/commands/mod.rs`

## References

- Plan Task 9
- Requirements §14, §17, §22
