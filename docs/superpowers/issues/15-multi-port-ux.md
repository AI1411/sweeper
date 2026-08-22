---
title: "[Post-MVP] Improve multi-port kill confirmation UX"
labels: [enhancement, post-mvp]
depends_on: []
priority: medium
---

## Summary

`sw :3000 :3001` already resolves multiple ports, but confirmation is per-PID in a loop. Improve the flow so multi-port cleanup feels like one intentional action.

## Motivation

Developers often free several related ports at once (API + Vite + Storybook). A single summary + batch confirm reduces friction without adding `-y`.

## Acceptance criteria

- [ ] `sw :3000 :3001` (and more) shows one summary table: port, PID, name, CPU/MEM
- [ ] User gets a clear confirm step (kill all matched / pick individually — at least one polished path)
- [ ] Duplicate PIDs listening on multiple selected ports are shown once
- [ ] `--force` still only affects signal escalation, not skipping confirm
- [ ] History records each kill with associated ports
- [ ] Tests cover de-duplication of PIDs across ports

## Suggested UX

```text
PORT   PID    PROCESS     MEM
3000   4812   node        210 MB
5173   4820   node        180 MB

Kill 2 processes? [y/N]
```

## References

- Requirements §23 Priority A (multiple port terminate)
- `src/commands/port.rs`
