---
title: "[Safety] --dry-run mode for kill commands"
labels: [enhancement, post-mvp]
depends_on: []
priority: medium
---

## Summary

Add `--dry-run` to show what would be killed (PIDs, names, ports, tree expansion) without sending signals.

## Motivation

Developers and scripts need to inspect kill targets safely. Aligns with “Sweeper proposes. User decides.” and enables safer automation (`sw :3000 --dry-run` before kill).

## Acceptance criteria

- [ ] Global or per-command flag: `--dry-run` on name, port, project, clean (and TUI optional later)
- [ ] Prints target PIDs, process names, ports, and tree descendants when `--tree` is set
- [ ] Skips protected processes with clear message
- [ ] No history entries written on dry-run
- [ ] No signals sent
- [ ] Documented in README and `--help`

## Suggested UX

```text
Dry run — would terminate 2 process(es):
  node  pid 4812  ports :3000
  vite  pid 1602  ports :5173  (tree child of 4812)
No signals sent.
```

## Implementation notes

- `src/cli.rs` — add flag
- Shared helper: `plan_kill(pids, tree) -> Vec<PlannedKill>` used by dry-run and real kill paths

## References

- Requirements §15–16, §25
- `src/commands/{name,port,clean,project}.rs`, `src/process/tree.rs`
