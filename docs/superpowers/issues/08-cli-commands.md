---
title: "[MVP] Task 8: Name / port / top CLI commands"
labels: [enhancement, mvp]
depends_on: [2, 4, 5, 6, 7]
plan: docs/superpowers/plans/2026-08-21-sweeper-mvp.md#task-8
---

## Summary

`sw <name>` / `sw :port` / `sw top` を実装。確認プロンプト付き。`-y` は作らない。

## Acceptance criteria

- [ ] `confirm(prompt) -> bool`（デフォルト N）
- [ ] `sw node` が候補表示 → 確認 → kill → history
- [ ] `sw :3000` がポート所有者を表示 → 確認 → kill
- [ ] StillAlive 時に Force? を聞ける（`--force` なら即 SIGKILL 経路）
- [ ] `sw top` が CPU / MEMORY 上位を表示
- [ ] `main` が `Cli.target` で dispatch

## Files

- Create: `src/commands/{mod,confirm,name,port,top}.rs`
- Modify: `src/main.rs`, `src/lib.rs`

## References

- Plan Task 8
- Requirements §7, §8, §13, §22
