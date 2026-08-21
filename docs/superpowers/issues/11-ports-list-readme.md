---
title: "[MVP] Task 11: sw ports list + README"
labels: [enhancement, mvp, documentation]
depends_on: [5, 8]
plan: docs/superpowers/plans/2026-08-21-sweeper-mvp.md#task-11
---

## Summary

LISTEN ポート一覧コマンドと README の使い方を追加し、MVP テストを通す。

## Acceptance criteria

- [ ] `sw ports` が PORT / PROCESS / PID 表を出す
- [ ] README に install（dev）と主要コマンド例
- [ ] `cargo test` 全 PASS

## Files

- Create: `src/commands/ports_list.rs`（または `port.rs` 拡張）
- Modify: `src/main.rs`, `README.md`

## References

- Plan Task 11
- Requirements §8.4
