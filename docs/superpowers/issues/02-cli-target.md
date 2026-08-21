---
title: "[MVP] Task 2: CLI target resolution"
labels: [enhancement, mvp]
depends_on: [1]
plan: docs/superpowers/plans/2026-08-21-sweeper-mvp.md#task-2
---

## Summary

`clap` で `sw <target>` を解決する。`Cli { force, tree, target }` を公開する。

## Acceptance criteria

- [ ] `sw` → `Target::Tui`
- [ ] `sw node` → `Target::Name`
- [ ] `sw :3000` / 複数ポート → `Target::Ports`
- [ ] `sw top|ports|clean|history|project` → `Target::Sub`
- [ ] `--force` / `--tree` が取れる
- [ ] `cargo test --test cli_target` が PASS

## Files

- Create: `src/cli.rs`, `tests/cli_target.rs`
- Modify: `src/lib.rs`, `src/main.rs`

## References

- Plan Task 2
- Requirements §5, §19
