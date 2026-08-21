---
title: "[MVP] Task 4: Process listing via sysinfo"
labels: [enhancement, mvp]
depends_on: [1]
plan: docs/superpowers/plans/2026-08-21-sweeper-mvp.md#task-4
---

## Summary

`sysinfo` でプロセス一覧を取得し、名前の曖昧検索を提供する。

## Acceptance criteria

- [ ] `list_processes() -> Vec<ProcessInfo>` が動作する
- [ ] `find_by_name_fuzzy(query)` が名前・コマンドに部分一致する
- [ ] CPU / memory / ppid / command / cwd を可能な範囲で埋める
- [ ] ports は空でよい（Task 5 で merge）

## Files

- Create: `src/process/list.rs`
- Modify: `src/process/mod.rs`

## References

- Plan Task 4
- Spec §5 process 層
