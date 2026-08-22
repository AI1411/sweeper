---
title: "[Testing] Golden tests for CLI output (NO_COLOR)"
labels: [enhancement, post-mvp]
depends_on: []
priority: low
---

## Summary

Fixture-based golden tests for stable CLI output of `sw clean`, `sw ports`, and kill summaries with `NO_COLOR=1`.

## Motivation

Colored, formatted output is easy to break silently. Golden files catch formatting regressions in reasons, summaries, and tables.

## Acceptance criteria

- [ ] Golden tests for `format_reasons_display`, kill summary lines, and at least one command output path
- [ ] Run with `NO_COLOR=1` or force plain style in test harness
- [ ] Fixture `ProcessInfo` data — no live process dependency
- [ ] `UPDATE_GOLDEN=1` or similar to refresh goldens when intentional
- [ ] CI runs golden tests

## Implementation notes

- `tests/golden_cli.rs` or extend `tests/clean_propose.rs`
- Compare normalized whitespace if needed

## References

- `src/style.rs`, `src/commands/clean.rs`, `src/report.rs`
