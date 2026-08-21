---
title: "[MVP] Task 3: Protected process list"
labels: [enhancement, mvp]
depends_on: [1]
plan: docs/superpowers/plans/2026-08-21-sweeper-mvp.md#task-3
---

## Summary

誤って重要プロセスを殺さないための保護リストを実装する。

## Acceptance criteria

- [ ] `is_protected(name)` が `kernel_task` / `launchd` / `WindowServer` 等を true
- [ ] `node` / `vite` 等は false
- [ ] `cargo test --test protect` が PASS

## Files

- Create: `src/process/protect.rs`, `tests/protect.rs`
- Modify: `src/process/mod.rs`

## References

- Plan Task 3
- Requirements §16
