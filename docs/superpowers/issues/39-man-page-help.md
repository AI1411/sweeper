---
title: "[Docs] Man page and enriched --help"
labels: [documentation, enhancement, post-mvp]
depends_on: []
priority: low
---

## Summary

Provide a man page (`man sw`) and richer clap help for TUI keys and safety rules.

## Motivation

TUI keybindings and safety rules live mainly in README. Terminal users expect `sw --help` and `man sw` for quick reference without opening the repo.

## Acceptance criteria

- [ ] `sw --help` lists all subcommands with examples in long help where useful
- [ ] Man page source (e.g. `docs/sw.1.md` or `man/sw.1`) covering commands, flags, TUI keys, safety
- [ ] README links to man install path (`cargo install` does not install man — document `make install` or brew)
- [ ] clap `about` / `long_about` aligned with README safety section

## Implementation notes

- `src/cli.rs` — clap metadata
- Optional: `clap_mangen` for auto man generation

## References

- `README.md`, `src/cli.rs`
