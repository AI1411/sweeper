---
title: "[MVP] Task 7: History store (JSON)"
labels: [enhancement, mvp]
depends_on: [1]
plan: docs/superpowers/plans/2026-08-21-sweeper-mvp.md#task-7
---

## Summary

kill 履歴を Application Support 配下の JSON に保存する（上限 200）。

## Acceptance criteria

- [ ] `HistoryEntry { time, pid, name, ports, signal, result }`
- [ ] デフォルトパス: `~/Library/Application Support/sweeper/history.json`（`directories`）
- [ ] `append_entry_at` / `load_entries_at` でテスト可能
- [ ] 200 件超で古いものから削除
- [ ] `cargo test --test history_store` が PASS

## Files

- Create: `src/history/mod.rs`, `tests/history_store.rs`
- Modify: `src/lib.rs`

## References

- Plan Task 7
- Spec §6 history
