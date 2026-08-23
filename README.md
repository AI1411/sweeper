# Sweeper

**Sweep unwanted processes away.**

A developer-focused CLI/TUI for finding leftover processes and terminating them safely. Primary target is macOS; Linux is formally supported for CLI, TUI, ports, clean, and history.

Binary name: `sw`

## Platform compatibility

| Feature | macOS | Linux |
| --- | --- | --- |
| TUI (`sw`) | Yes | Yes |
| Name / port search | Yes | Yes |
| `sw ports` / `sw top` | Yes | Yes |
| `sw clean` / `sw history` | Yes | Yes |
| `sw project` | Yes | Yes |
| Native port lookup | libproc (lsof fallback) | `/proc/net/tcp` (lsof fallback) |
| History file | `~/Library/Application Support/com.sweeper.sweeper/history.json` | `~/.local/share/com.sweeper.sweeper/history.json` (XDG) |
| Protect config | `~/Library/Application Support/com.sweeper.sweeper/protect.toml` | `~/.config/sweeper/protect.toml` (XDG) |
| Built-in protect list | macOS system daemons | Linux system daemons (`systemd`, `sshd`, …) |

## Install

From source:

```bash
cargo install --path .
```

From a release tag:

```bash
cargo install --git https://github.com/AI1411/sweeper --tag v0.1.0
```

Prebuilt binaries for Linux (x86_64) and macOS (arm64 / x86_64) are attached to [GitHub Releases](https://github.com/AI1411/sweeper/releases). Download the archive for your platform, extract `sw`, and place it on your `PATH`.

Man page source: [`docs/sw.1.md`](docs/sw.1.md) (install manually to your `MANPATH` if desired).

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
sw project      # list inferred projects
sw project app  # inspect / kill a project
```

## Commands

| Command | Alias | Description |
| --- | --- | --- |
| `sw` | — | Interactive TUI process browser |
| `sw <name>` | — | Fuzzy search by process name |
| `sw :<port>` | — | Find process(es) by port (repeatable) |
| `sw ports` | `p` | List LISTEN ports with process and PID |
| `sw top` | `t` | Top processes by CPU and memory; interactive kill by rank or PID |
| `sw clean` | `c` | Propose leftover candidates; confirm before kill |

`sw clean --exclude <pattern>` skips candidates whose name or PID contains the pattern (repeatable).  
Also honors `SWEEPER_CLEAN_EXCLUDE` (comma-separated).
| `sw history` | `h` | Kill history (`--last` for one entry) |
| `sw project` | `proj` | List inferred projects; `sw project <name>` to kill |

Examples:

```bash
sw node
sw :3000 :3001
sw :3000-3010
sw ports
sw top          # then enter rank (1-10), memory rank (m1-m10), or PID to kill
sw clean
sw history --last
sw project
sw project my-app
```

Scripting with JSON:

```bash
sw ports --json | jq '.[] | select(.port == 3000)'
sw clean --json | jq '.candidates[].pid'
sw project --json | jq '.[].name'
sw history --json | jq '.[-1].pid'
```

## Options

| Flag | Description |
| --- | --- |
| `--force` | Allow SIGKILL when a process does not exit after SIGTERM |
| `--tree` | Also kill descendants (PPID tree), children first |
| `--dry-run` | Show kill targets without sending signals |
| `--json` | Machine-readable JSON output (`ports`, `clean`, `project`, `history`) |
| `-h`, `--help` | Print help |

Flags may appear before or after targets:

```bash
sw :3000 --force
sw --force node
sw node --tree
```

## TUI

Launch with bare `sw`.

| Key | Action |
| --- | --- |
| `↑` / `↓` | Move |
| `g` / `G` | Jump to first / last row |
| `PgUp` / `PgDn` | Page up / down |
| `Ctrl-u` / `Ctrl-d` | Page up / down |
| `Space` | Select / deselect |
| `/` | Search |
| `k` | Preview SIGTERM selected; `y` confirms |
| `K` | Preview SIGKILL selected; `y` confirms |
| `t` | Preview SIGTERM + descendants; `y` confirms |
| `T` | Preview SIGKILL + descendants; `y` confirms |
| `y` / `n` / `Esc` | Confirm or cancel pending kill |
| `p` | Toggle “listening ports only” filter |
| `P` | Toggle project grouping view |
| `e` | Toggle process tree view |
| `i` / `Enter` | Toggle process detail panel (Enter expands project in project view) |
| `r` | Refresh processes and ports |
| `q` | Quit |

## Safety

- Default signal is **SIGTERM**. **SIGKILL** only with `--force` or TUI `K`.
- Critical system processes are protected from kill (OS-specific built-in list).
- `sw clean` never auto-kills — it proposes; you decide.
- `sw clean` skips active dev servers; it flags orphans, stale/idle listeners, and zombies.
- There is no `-y` / `--yes` skip for confirmations.
- After kills, Sweeper prints an **estimated** memory freed total from the pre-kill snapshot (not proof of OS reclaim).

## Protected processes

Built-in protection covers critical system daemons on macOS and Linux. Extend the list with a config file:

```text
# ~/.config/sweeper/protect.toml  (Linux)
# ~/Library/Application Support/com.sweeper.sweeper/protect.toml  (macOS)

postgres
redis
my-critical-daemon
```

One process name per line; `#` comments allowed. Protected names are matched case-insensitively against the process basename.

## Colors

CLI and TUI use color when stdout is a TTY. Set `NO_COLOR` to disable.

## Development

Port lookup uses native OS APIs by default (`/proc/net/tcp` on Linux, `libproc` on macOS). If native lookup fails, Sweeper falls back to `lsof` and prints a one-line note.

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
```
