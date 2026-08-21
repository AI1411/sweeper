---
title: "[MVP] Task 1: Cargo scaffold + error types + ProcessInfo"
labels: [enhancement, mvp]
depends_on: []
plan: docs/superpowers/plans/2026-08-21-sweeper-mvp.md#task-1
---

## Summary

Rust プロジェクトを scaffold し、共通エラー型と `ProcessInfo` を定義する。

## Acceptance criteria

- [ ] `Cargo.toml` に clap / ratatui / sysinfo / nix / serde 等が定義されている
- [ ] バイナリ名が `sw`（`[[bin]] name = "sw"`）
- [ ] `SweeperError` / `Result<T>` がある
- [ ] `ProcessInfo { pid, ppid, name, cpu, memory_bytes, ports, command, cwd }` がある
- [ ] `cargo build` が通る

## Files

- Create: `Cargo.toml`, `src/main.rs`, `src/lib.rs`, `src/error.rs`, `src/process/mod.rs`, `src/process/types.rs`

## References

- Spec: `docs/superpowers/specs/2026-08-21-tech-stack-design.md`
- Plan Task 1: `docs/superpowers/plans/2026-08-21-sweeper-mvp.md`
