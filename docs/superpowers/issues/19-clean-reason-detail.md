---
title: "[Quality] Human-readable sw clean reasons (age, port, command)"
labels: [enhancement, post-mvp]
depends_on: []
priority: high
---

## Summary

Make `sw clean` candidates explain themselves in **plain language** — not only reason tags like `stale-server`, but concrete context: uptime, ports, and a short command snippet.

## Motivation

Reason flags (#37) build trust, but opaque tags still force users to infer context. Quality = **actionable explanations** so users can confirm kills without `ps` / `lsof`.

## Acceptance criteria

- [ ] Each candidate line includes human-readable reason detail derived from snapshot data
- [ ] Examples:
  - `stale-server` → `stale-server (5h on :3000)`
  - `idle-listener` → `idle-listener (45m, CPU 0.0%)`
  - `orphan-parent` → `orphan-parent (ppid 9999 missing)`
  - `stack:vite` → show first ~40 chars of command when available
- [ ] Optional confidence hint (high / medium) based on score — or sort by score without new UI chrome
- [ ] Output stays colorful and scannable (`style` helpers)
- [ ] Never auto-kills; confirm flow unchanged
- [ ] Unit tests in `tests/clean_propose.rs` for formatted reason strings

## Suggested UX

```text
Sweeper found possible leftovers:

✓ 2 candidate processes
✓ 1 stale dev server
Estimated memory reclaim: 312 MB

  node  pid 1513  ports [3000]  age 5h
    reasons: stack:vite, stale-server (5h on :3000), listening
    cmd: node ./node_modules/.bin/vite --port 3000

  vite  pid 1602  ports []  age 12m
    reasons: orphan-ppid, stack:vite
```

## Implementation notes

- Add `format_reasons(candidate: &CleanCandidate) -> Vec<String>` in `src/clean/mod.rs`
- Reuse `run_time_secs`, `ports`, `command`, `ppid` on `ProcessInfo`
- Keep machine-readable tags internally; pretty-print only in CLI output

## References

- Requirements §14 (`sw clean`), §26 (framework-aware cleanup direction)
- `src/clean/mod.rs`, `src/commands/clean.rs`
