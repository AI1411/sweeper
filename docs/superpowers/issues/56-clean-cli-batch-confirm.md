---
title: "[Clean] CLI batch confirm for sw clean"
labels: [enhancement, post-mvp]
depends_on: []
priority: high
---

## Summary

Improve `sw clean` CLI flow so users can confirm kills in one batch step (with optional per-process opt-out), matching the TUI Clean view UX.

## Motivation

`sw clean` currently asks for confirmation on **every** candidate individually after the initial prompt. Cleaning 10+ leftovers is tedious compared to the TUI flow (`Space` select → `k` → `y`).

## Acceptance criteria

- [ ] After showing the sorted candidate list, offer a batch confirm: `Kill N processes? [y/N]`
- [ ] Optional: allow selecting by number (`1,3,5`) or `--include` / `--exclude` flags
- [ ] `--force` still only affects signal escalation, not skipping confirm
- [ ] No auto-kill; user must explicitly confirm
- [ ] History records each kill with ports and signal
- [ ] Tests cover batch confirm, partial selection, and empty selection

## Suggested UX

```text
Sweeper found 4 possible leftovers (sorted by confidence):
...

Kill all 4 processes? [y/N]
> n

Kill which? (comma-separated numbers, or 'q' to quit): 1,3
Kill node (pid 4812)? [y/N]
Kill vite (pid 5120)? [y/N]
```

## Implementation notes

- `src/commands/clean.rs` — refactor confirm loop
- Reuse TUI clean selection logic where possible

## References

- `src/commands/clean.rs`
- TUI Clean view: `src/tui/app.rs`, `src/tui/mod.rs`
