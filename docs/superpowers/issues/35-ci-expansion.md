---
title: "[CI] Expand quality gates (doc, audit, matrix)"
labels: [enhancement, post-mvp]
depends_on: [30-macos-ci]
priority: low
---

## Summary

Strengthen CI beyond test/fmt/clippy: `cargo doc`, dependency audit, and coordinated OS matrix.

## Motivation

Catch doc breakage, known CVEs in deps, and cross-platform issues before release.

## Acceptance criteria

- [ ] `cargo doc --no-deps` job (warnings as errors optional)
- [ ] `cargo audit` or `deny` in CI (document allowlist if needed)
- [ ] Matrix or parallel jobs for Linux + macOS (depends on #30)
- [ ] Failed audit does not block on unfixable upstream CVE without documented exception

## Implementation notes

- `.github/workflows/ci.yml`
- `cargo install cargo-audit` in audit job

## References

- `.github/workflows/ci.yml`
