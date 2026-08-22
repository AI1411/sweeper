---
title: "[CI] macOS GitHub Actions workflow"
labels: [enhancement, post-mvp]
depends_on: []
priority: high
---

## Summary

Run `cargo test`, `fmt`, and `clippy` on macOS in CI — Sweeper’s primary target platform.

## Motivation

Current CI only uses `ubuntu-latest`. `lsof` output, paths, protected process names, and Application Support paths differ on macOS. Linux-green does not guarantee macOS correctness.

## Acceptance criteria

- [ ] `.github/workflows/ci.yml` includes `macos-latest` (or `macos-14`) job(s)
- [ ] Same gates as Linux: `cargo test --all-targets`, `cargo fmt --check`, `clippy -D warnings`
- [ ] Document any macOS-only skipped tests with reason
- [ ] Workflow passes on main after merge

## Implementation notes

- Mirror existing `test` and `lint` jobs with `runs-on: macos-latest`
- `Swatinem/rust-cache` works on macOS

## References

- `.github/workflows/ci.yml`
- Requirements §4 (macOS primary)
