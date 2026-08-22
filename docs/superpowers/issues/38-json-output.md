---
title: "[Feature] --json output mode for scripting"
labels: [enhancement, post-mvp]
depends_on: []
priority: medium
---

## Summary

Add `--json` flag for machine-readable output on `sw ports`, `sw clean`, `sw project`, and kill summaries.

## Motivation

Agents and scripts need structured data without parsing colored tables. Improves Sweeper as a dev-environment tool in automation.

## Acceptance criteria

- [ ] Global `--json` on relevant subcommands (at least `ports`, `clean`, `project`, `history`)
- [ ] JSON schema stable fields: pid, name, ports, memory_bytes, reasons (clean), project groups
- [ ] `NO_COLOR` implied or colors disabled when `--json`
- [ ] Errors as JSON on stderr or structured exit (document convention)
- [ ] README examples for `jq` pipelines
- [ ] Unit tests parse emitted JSON

## Suggested UX

```bash
sw ports --json | jq '.[] | select(.port == 3000)'
sw clean --json | jq '.candidates[].pid'
```

## Implementation notes

- `serde_json` already in dependencies
- `src/cli.rs` global flag; commands branch on format

## References

- Requirements §25 (scriptable dev workflow)
- `src/commands/{ports_list,clean,project,history}.rs`
