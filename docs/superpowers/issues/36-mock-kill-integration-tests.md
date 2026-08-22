---
title: "[Testing] Mock kill layer for integration tests"
labels: [enhancement, post-mvp]
depends_on: []
priority: low
---

## Summary

Introduce a mockable kill interface so kill flow integration tests run without sending real signals.

## Motivation

Kill flow tests today avoid real kills or only test pure helpers. A mock layer enables end-to-end CLI dry-run and confirm flow tests safely.

## Acceptance criteria

- [ ] `kill_pid` behind trait or test-only injectable function
- [ ] Integration test: port target → confirm → mock kill → history entry + summary
- [ ] Production binary uses real `nix` signals; tests use mock
- [ ] No real `kill()` in `cargo test` default suite

## Implementation notes

- `src/process/kill.rs` — `KillFn` or `#[cfg(test)]` hook
- Complements `--dry-run` issue (#24)

## References

- `src/process/kill.rs`, `src/commands/port.rs`
