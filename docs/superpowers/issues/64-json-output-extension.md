---
title: "[Feature] Extend --json to sw top and process listing"
labels: [enhancement, post-mvp]
depends_on: [38-json-output]
priority: low
---

## Summary

Add `--json` output to `sw top` and optionally a machine-readable process list for scripting.

## Motivation

`ports`, `clean`, `project`, and `history` support `--json`, but `sw top` and bare process queries do not — limiting automation and dashboards.

## Acceptance criteria

- [ ] `sw top --json` emits CPU and memory leader arrays with pid, name, cpu, memory_bytes
- [ ] Schema documented in README and `src/json_output.rs`
- [ ] Stable field names (snake_case, version field optional)
- [ ] `NO_COLOR` / non-TTY behavior unchanged for plain output
- [ ] Golden or unit test for JSON shape

## Example

```json
{
  "cpu": [{"rank": 1, "pid": 123, "name": "node", "cpu": 42.1, "memory_bytes": 210000000}],
  "memory": [{"rank": 1, "pid": 456, "name": "Docker", "cpu": 1.2, "memory_bytes": 4200000000}]
}
```

## Implementation notes

- `src/commands/top.rs`, `src/json_output.rs`

## References

- Related: #72 (`--json` output mode)
