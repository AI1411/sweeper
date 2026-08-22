---
title: "[UX] Interactive kill from sw top"
labels: [enhancement, post-mvp]
depends_on: []
priority: medium
---

## Summary

Let users select and kill processes from `sw top` output, not only view CPU/memory leaders.

## Motivation

Requirements §13: “ここから対象を選択して終了できる.” Today `sw top` only prints two static lists.

## Acceptance criteria

- [ ] After top lists, prompt to pick a rank (or PID) to kill, or offer numbered selection
- [ ] Respects `--force`, `--tree`, protect list, and confirm flow
- [ ] Kill summary (memory freed, ports released) after success
- [ ] Empty / invalid selection handled gracefully
- [ ] Document in README

## Suggested UX

```text
CPU top 3 shown...
Kill by rank? [1-10 / q]
> 2
Kill node (pid 48291)? [y/N]
```

## Implementation notes

- `src/commands/top.rs` — interactive loop or sub-prompt
- Reuse `kill_pid`, `report::print_kill_summary`

## References

- Requirements §13 (`sw top`)
- `src/commands/top.rs`
