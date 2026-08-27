---
title: "[Performance] HashMap-based merge_ports"
labels: [enhancement, post-mvp]
depends_on: []
priority: medium
---

## Summary

Replace O(ports × processes) linear search in `merge_ports` with a PID-indexed HashMap.

## Motivation

`merge_ports` calls `procs.iter_mut().find(|p| p.pid == *pid)` for every port binding. With hundreds of processes and dozens of listeners, this adds unnecessary cost on every TUI port load and CLI command.

## Acceptance criteria

- [ ] Build `HashMap<u32, &mut ProcessInfo>` once, merge all ports in O(n)
- [ ] Behavior identical to current (multiple ports per PID, dedup)
- [ ] Unit test in `tests/merge_ports.rs` or inline module test
- [ ] No allocation regression for empty port lists

## Implementation notes

- `src/process/ports.rs` — `merge_ports`

## References

- `src/process/ports.rs` (`merge_ports`)
