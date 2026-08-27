---
title: "[Performance] Batch port lookup via LISTEN cache for multi-port CLI"
labels: [enhancement, post-mvp]
depends_on: [41-port-cache]
priority: medium
---

## Summary

When resolving multiple ports (`sw :3000 :3001`), fetch the full LISTEN table once and filter locally instead of calling `pids_for_port` per port.

## Motivation

Port cache (3s TTL) exists for `listening_ports()`, but `pids_for_port(3000)` bypasses it and may invoke native lookup or `lsof` per port. Multi-port kills are slower than necessary.

## Acceptance criteria

- [ ] `collect_unique_targets` uses cached LISTEN map when resolving multiple ports
- [ ] Single-port `sw :3000` behavior unchanged (may still use targeted lookup)
- [ ] `--no-cache` or `r` in TUI bypasses cache as today
- [ ] Test: second multi-port call within TTL does not spawn extra `lsof` (mock Command)
- [ ] Document in `src/process/ports.rs`

## Implementation notes

- `src/process/ports.rs` — `pids_for_ports(ports: &[u16])` using `listening_ports_cached`
- `src/commands/port.rs` — use batch helper

## References

- `src/process/ports.rs`, `src/commands/port.rs`
- Related: #75 (port cache TTL — implemented)
