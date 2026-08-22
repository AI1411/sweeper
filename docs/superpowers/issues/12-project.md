---
title: "[Post-MVP] sw project: group and kill by project"
labels: [enhancement, post-mvp]
depends_on: []
priority: high
---

## Summary

Implement `sw project` / `sw proj` (currently a stub). Infer which development project each process belongs to from working directory, command line, and parent/child relationships, then list and kill by project.

## Motivation

Core differentiator in the requirements (`docs/requirements.md` §11). CLI already accepts `project` / `proj` but prints `Not implemented yet`.

## Acceptance criteria

- [ ] `sw project` lists inferred projects with process counts, memory, and key ports
- [ ] `sw project <name>` shows that project's process tree / members
- [ ] User can confirm and kill an entire project (SIGTERM first; `--force` for SIGKILL)
- [ ] Inference uses at least cwd and/or command path (e.g. `~/dev/my-app`)
- [ ] Protected processes are never killed
- [ ] README documents usage
- [ ] Unit tests cover grouping heuristics with fixture `ProcessInfo` values

## Suggested UX

```bash
sw project
sw project my-app
sw proj my-app --force
```

## References

- Requirements §11, §23 Priority B
- `src/main.rs` (`SubCommand::Project` stub)
- `src/cli.rs` (`proj` alias)
