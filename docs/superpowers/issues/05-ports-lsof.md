---
title: "[MVP] Task 5: Port resolution via lsof"
labels: [enhancement, mvp]
depends_on: [4]
plan: docs/superpowers/plans/2026-08-21-sweeper-mvp.md#task-5
---

## Summary

`lsof` 出力をパースして LISTEN ポートと PID を解決し、`ProcessInfo` に merge する。

## Acceptance criteria

- [ ] `parse_lsof_listen_line` が典型行を `(pid, port)` にできる
- [ ] ESTABLISHED 行は無視する
- [ ] `listening_ports()` / `pids_for_port(port)` が動く
- [ ] `merge_ports(procs, port_map)` がある
- [ ] `cargo test --test ports_parse` が PASS

## Files

- Create: `src/process/ports.rs`, `tests/ports_parse.rs`
- Modify: `src/process/mod.rs`

## References

- Plan Task 5
- Requirements §8
