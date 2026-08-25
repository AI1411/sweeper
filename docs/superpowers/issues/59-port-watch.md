---
title: "[Feature] sw watch :port — wait for port state changes"
labels: [enhancement, post-mvp]
depends_on: []
priority: medium
---

## Summary

Add `sw watch :<port>` (and optional port ranges) to poll a port until it becomes **LISTEN** or **free**, printing updates at a configurable interval. Replaces the common `while lsof ...; do sleep 1; done` loop when waiting for dev servers to start or shut down.

## Motivation

Developers frequently:

- Start a server and wait until `:3000` is listening
- Kill a process and wait until the port is released before restarting
- Script CI/local flows that depend on port availability

Sweeper already resolves ports natively; a watch mode completes the `lsof` + `kill` workflow without leaving the `sw` CLI.

## Acceptance criteria

### CLI behavior

- [ ] `sw watch :3000` — poll until port has a LISTEN process (default mode: `wait-listen`)
- [ ] `sw watch :3000 --until free` (or `--free`) — poll until no listener on port
- [ ] `--interval <secs>` default 1s; minimum 0.5s documented
- [ ] On success: print PID/process (listen mode) or confirmation (free mode); exit 0
- [ ] On `--timeout <secs>`: exit non-zero with clear message
- [ ] Support multiple ports: `sw watch :3000 :3001` — document whether all must match or any (recommend: all for listen, all free for free mode)
- [ ] Port ranges reuse existing `:3000-3010` parsing (#60)
- [ ] `--json` emits events: `{ "event": "listening", "port": 3000, "pid": 1234, "process": "node" }` on transition

### UX

- [ ] Quiet mode `--quiet` — only print final line (for scripts)
- [ ] Show spinner or timestamped lines when not `--quiet` (match Sweeper style)
- [ ] Ctrl-C exits 130 with no kill side effects

### Safety

- [ ] Read-only: watch never sends signals
- [ ] Reuse port resolution stack (native + lsof fallback); note fallback in verbose output

### Tests

- [ ] Unit tests for watch state machine (mock port resolver)
- [ ] Integration test with ephemeral bind/unbind in CI if feasible; otherwise mock-only

## Suggested usage

```bash
# Wait for dev server
sw watch :3000

# Wait until port is free after kill
sw kill :3000 && sw watch :3000 --until free

# Script with timeout
sw watch :5173 --timeout 60 --quiet
```

```text
$ sw watch :3000
watching :3000 (wait for LISTEN, interval 1s) …
… :3000 free
✓ :3000 listening — node pid 48291
```

## Implementation notes

- `src/commands/watch.rs` (new) or extend `src/commands/port.rs`
- `src/cli.rs` — parse `watch` as subcommand or positional alias; prefer explicit `sw watch :3000` to avoid ambiguity with `sw :3000` kill flow
- Reuse `src/ports/` resolution; consider sharing TTL cache (#75) with care (watch may need fresh reads)
- Distinct from `sw memory watch` (#113) — different domain; document in README

## References

- Requirements §8 (ポート検索)
- Closed: #60 (port range), #65 (native port resolution)
- `src/commands/port.rs`, `src/ports/`

## Related

- #75 (port cache) — watch should bypass or shorten TTL while watching
