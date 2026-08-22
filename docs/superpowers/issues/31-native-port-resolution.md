---
title: "[Platform] Native port resolution (reduce lsof dependency)"
labels: [enhancement, post-mvp]
depends_on: []
priority: medium
---

## Summary

Replace or supplement `lsof` subprocess calls with native APIs for LISTEN port discovery on macOS (and optionally Linux).

## Motivation

Tech stack design allows `lsof` for MVP but notes native replacement. `lsof` is slow on large systems, may be missing, and parses fragile text output.

## Acceptance criteria

- [ ] macOS: resolve LISTEN ports without `lsof` for default code path (libproc / netstat API — pick one documented approach)
- [ ] Fallback to `lsof` when native path fails, with one-line user-visible hint
- [ ] `listening_ports()` and `pids_for_port()` behavior unchanged for callers
- [ ] Existing `parse_lsof_listen_line` tests remain; add native parser tests with fixture lines or mock data
- [ ] README notes dependency change

## Implementation notes

- `src/process/ports.rs` — trait or `#[cfg(target_os)]` modules
- Keep `merge_ports` API stable

## References

- `docs/superpowers/specs/2026-08-21-tech-stack-design.md` §3
- `src/process/ports.rs`, `tests/ports_parse.rs`
