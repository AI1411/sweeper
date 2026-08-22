---
title: "[Distribution] GitHub Releases and binary artifacts"
labels: [enhancement, post-mvp]
depends_on: []
priority: medium
---

## Summary

Automate versioned releases with prebuilt binaries for macOS (arm64/x64) and Linux, plus `cargo install` from registry or git tag.

## Motivation

Today only `cargo install --path .` is documented. Homebrew and direct binary download were planned in tech stack; releases unblock adoption.

## Acceptance criteria

- [ ] Tag push (e.g. `v0.2.0`) triggers release workflow
- [ ] Artifacts: macOS arm64 + Linux x64 minimum (`sw` binary)
- [ ] CHANGELOG or release notes template
- [ ] README install section: releases URL + `cargo install sweeper --git …`
- [ ] Optional follow-up: Homebrew tap (separate issue or sub-task)

## Implementation notes

- `.github/workflows/release.yml` with `cross` or native runners
- Bump `Cargo.toml` version on release

## References

- `docs/superpowers/specs/2026-08-21-tech-stack-design.md` §3 (Homebrew)
- `README.md`, `Cargo.toml`
