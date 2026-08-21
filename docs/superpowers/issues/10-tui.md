---
title: "[MVP] Task 10: TUI (ratatui)"
labels: [enhancement, mvp]
depends_on: [8]
plan: docs/superpowers/plans/2026-08-21-sweeper-mvp.md#task-10
---

## Summary

`sw` 単体で TUI を起動。一覧・検索・複数選択・kill。

## Acceptance criteria

- [ ] 列: PID / PROCESS / PORT / CPU / MEM
- [ ] `/` 検索、Space 選択、`k` SIGTERM、`K` SIGKILL、`q` 終了
- [ ] ポートは `std::thread` + channel で後から merge（tokio 不使用）
- [ ] `Target::Tui => tui::run()`
- [ ] view と state を分離（`app.rs` / `ui.rs`）

## Files

- Create: `src/tui/{mod,app,ui}.rs`
- Modify: `src/main.rs`, `src/lib.rs`

## References

- Plan Task 10
- Requirements §6, §9, §20, §22
