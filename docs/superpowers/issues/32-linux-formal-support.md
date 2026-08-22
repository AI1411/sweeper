---
title: "[Platform] Formal Linux support and compatibility matrix"
labels: [enhancement, post-mvp]
depends_on: []
priority: medium
---

## Summary

Document and harden Linux support: OS-specific protect list, paths, and a README compatibility table.

## Motivation

README says “most commands also work on Linux” but protect list and path heuristics are macOS-centric. Formal support reduces surprise kills and bad project inference.

## Acceptance criteria

- [ ] README compatibility table: feature × macOS × Linux (TUI, ports, clean, history path, etc.)
- [ ] Linux-relevant protected process names (e.g. `systemd`, `sshd`) in protect list or OS-specific section
- [ ] `project` path heuristics skip Linux system paths (`/usr`, `/snap`, etc.) — audit `src/project/mod.rs`
- [ ] History/config paths work on Linux via `directories` crate (XDG)
- [ ] CI Linux job remains green; no macOS-only assumptions in shared code without `cfg`

## Implementation notes

- `src/process/protect.rs`, `src/project/mod.rs`, `README.md`
- Optional: `#[cfg(target_os = "linux")]` modules

## References

- Requirements §4 (Linux future), Priority C
- `README.md`, `src/project/mod.rs`
