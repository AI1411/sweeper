---
title: "[Feature] Port range search sw :3000-3010"
labels: [enhancement, post-mvp]
depends_on: []
priority: medium
---

## Summary

Support port range syntax so users can find and kill processes listening on consecutive ports in one command.

## Motivation

Requirements §8.3. Typical dev setups use multiple adjacent ports (3000–3010). Repeating `:3000 :3001 …` is tedious.

## Acceptance criteria

- [ ] CLI accepts `sw :3000-3010` (inclusive range)
- [ ] Range expands to individual ports; dedupe PIDs across range
- [ ] Invalid range (`:3010-3000`, port > 65535) returns clear error
- [ ] Works alongside multiple targets: `sw :3000-3002 :5173`
- [ ] Target resolution tests in `tests/cli_target.rs`
- [ ] README example

## Implementation notes

- `src/cli.rs` / target parser — extend port token parsing
- Reuse multi-port UX from issue #36 (dedupe, one confirm)

## References

- Requirements §8.3 (ポート範囲)
- `src/cli.rs`, `src/commands/port.rs`, `tests/cli_target.rs`
