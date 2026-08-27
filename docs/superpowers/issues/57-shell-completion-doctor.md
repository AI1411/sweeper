---
title: "[DX] Shell completions and sw doctor diagnostics"
labels: [enhancement, post-mvp]
depends_on: []
priority: high
---

## Summary

Add shell tab-completion for `sw` and a `sw doctor` command that checks common setup problems (permissions, missing tools, config paths, port-resolution fallbacks).

## Motivation

New users face friction discovering subcommands, ports, and project names. When port lookup or kill fails, there is no guided troubleshooting path — users must infer whether `lsof`, permissions, or OrbStack/Docker state is the cause.

Lowering onboarding and support burden improves adoption without changing core kill safety.

## Acceptance criteria

### Shell completions

- [ ] `sw completions bash` / `zsh` / `fish` subcommand (or `sw --generate-completion <shell>` — pick one, document)
- [ ] Complete: subcommands (`ports`, `top`, `clean`, `history`, `project`, `memory`, `docker`, `disk`, `cache`)
- [ ] Complete: global flags (`--force`, `--tree`, `--dry-run`, `--json`)
- [ ] Complete: positional targets where feasible (e.g. `:port` pattern hint; project names from last `sw project` cache or live scan — document performance tradeoff)
- [ ] Install instructions in README (one-liner per shell)
- [ ] Test: snapshot or unit test that completion script is non-empty and contains expected subcommands

### `sw doctor`

- [ ] `sw doctor` runs a checklist and prints pass/warn/fail per check
- [ ] Checks (best-effort, platform-aware):
  - [ ] Binary runs and version prints
  - [ ] Port resolution: native path works; note if falling back to `lsof`
  - [ ] `lsof` on PATH when native resolution unavailable
  - [ ] History / protect config paths writable and readable
  - [ ] OrbStack/Docker CLI reachable when relevant (`orb`, `docker`) — warn only, not fail
  - [ ] TTY / color support note when stdout is not a terminal
- [ ] Exit code 0 if no failures; non-zero if any hard fail (document which checks are hard vs warn)
- [ ] `--json` output with `{ "checks": [{ "name", "status", "message" }] }`
- [ ] Mention `sw doctor` in `--help` long about and README troubleshooting section

## Suggested output

```text
$ sw doctor
Sweeper doctor

  ✓  sw binary (0.1.0)
  ✓  native port lookup (/proc/net/tcp)
  ✓  history writable (~/.local/share/com.sweeper.sweeper/history.json)
  ⚠  lsof not found (fallback unavailable if native lookup fails)
  ✓  protect config readable (~/.config/sweeper/protect.toml)

1 warning, 0 failures
```

## Implementation notes

- `clap_complete` crate for completions; wire in `src/cli.rs` + `src/main.rs`
- `src/commands/doctor.rs` (new) — pluggable `DiagnosticCheck` trait
- `src/commands/mod.rs` — register subcommand
- Platform modules: `src/ports/`, `src/history/`, `src/process/protect.rs`

## References

- Requirements §5 (CLI), README Install / Development
- Similar: `gh doctor`, `brew doctor`

## Related

- #65 (native port resolution) — doctor should reflect native vs lsof state
- #73 (man page) — link doctor from docs when available
