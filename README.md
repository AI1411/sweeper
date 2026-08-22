# Sweeper

**Sweep unwanted processes away.**

A developer-focused CLI/TUI for finding leftover processes and terminating them safely. Built for macOS; most commands also work on Linux.

Binary name: `sw`

## Install

```bash
cargo install --path .
```

## Quick start

```bash
sw              # open the TUI process browser
sw node         # find processes by name
sw :3000        # find whatever is listening on a port
sw p            # list listening ports
sw t            # show CPU / memory leaders
sw c            # propose leftovers (you confirm each kill)
sw h            # show kill history
sw h --last     # show the most recent kill
```

## Commands

| Command | Alias | Description |
| --- | --- | --- |
| `sw` | — | Interactive TUI process browser |
| `sw <name>` | — | Fuzzy search by process name |
| `sw :<port>` | — | Find process(es) by port (repeatable) |
| `sw ports` | `p` | List LISTEN ports with process and PID |
| `sw top` | `t` | Top processes by CPU and memory |
| `sw clean` | `c` | Propose leftover candidates; confirm before kill |
| `sw history` | `h` | Kill history (`--last` for one entry) |
| `sw project` | `proj` | Not implemented yet |

Examples:

```bash
sw node
sw :3000 :3001
sw ports
sw top
sw clean
sw history --last
```

## Options

| Flag | Description |
| --- | --- |
| `--force` | Allow SIGKILL when a process does not exit after SIGTERM |
| `--tree` | Accepted in MVP; tree kill behavior is still a stub |
| `-h`, `--help` | Print help |

Flags may appear before or after targets:

```bash
sw :3000 --force
sw --force node
```

## TUI

Launch with bare `sw`.

| Key | Action |
| --- | --- |
| `↑` / `↓` | Move |
| `Space` | Select / deselect |
| `/` | Search |
| `k` | SIGTERM selected |
| `K` | SIGKILL selected |
| `r` | Refresh |
| `q` | Quit |

## Safety

- Default signal is **SIGTERM**. **SIGKILL** only with `--force` or TUI `K`.
- Critical macOS process names are protected from kill.
- `sw clean` never auto-kills — it proposes; you decide.
- There is no `-y` / `--yes` skip for confirmations.

## Colors

CLI and TUI use color when stdout is a TTY. Set `NO_COLOR` to disable.

## Development

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
```
