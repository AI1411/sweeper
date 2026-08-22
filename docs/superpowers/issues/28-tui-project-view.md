---
title: "[UX] TUI project grouping view"
labels: [enhancement, post-mvp]
depends_on: []
priority: medium
---

## Summary

Add a TUI mode to group processes by inferred project (like `sw project`) with filter and kill by group.

## Motivation

CLI `sw project` exists but TUI users cannot browse or kill by project without leaving the TUI. Project recognition is a key differentiator.

## Acceptance criteria

- [ ] Key (e.g. `P` or `g` with modifier — avoid conflict with `g` jump) toggles project-grouped view
- [ ] Groups show name, path, process count, memory total, ports
- [ ] Expand group to see members; kill selection within group or whole group with confirm
- [ ] Reuses `src/project/mod.rs` grouping logic
- [ ] Footer documents keybinding
- [ ] Tests for group rendering data (pure helpers)

## Suggested UX

```text
Projects:
> my-app     ~/dev/my-app    4 processes  812 MB  :3000 :5173
  api-server ~/dev/api       2 processes  180 MB  :8787
```

## Implementation notes

- `src/tui/app.rs` — view mode enum (`List` | `Projects`)
- `src/project/mod.rs` — `group_by_project`

## References

- Requirements §11 (プロジェクト認識), §20 (`g` project — reconcile key with navigation `g`)
- `src/project/mod.rs`, `src/commands/project.rs`
