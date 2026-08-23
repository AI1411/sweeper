# SW(1) — Sweeper

## NAME

sw — sweep unwanted processes away

## SYNOPSIS

**sw** [*options*]  
**sw** *name* [*options*]  
**sw** *:port* [*options*]  
**sw** *subcommand* [*options*]

## DESCRIPTION

Sweeper is a developer-focused CLI and TUI for finding leftover dev processes and terminating them safely after explicit confirmation.

Default signal is SIGTERM. SIGKILL requires **--force** or TUI `K` after confirmation.

## COMMANDS

**ports** (**p**)  
List TCP LISTEN ports with process name and PID.

**top** (**t**)  
Show CPU and memory leaders; interactive kill by rank or PID.

**clean** (**c**)  
Propose leftover dev candidates (orphans, stale listeners, zombies). Never auto-kills.

**history** (**h**)  
Show kill history. **--last** prints one entry.

**project** (**proj**)  
List inferred projects from cwd/command. **sw project** *name* inspects or kills a project.

## OPTIONS

**--force**  
Allow SIGKILL when SIGTERM is insufficient.

**--tree**  
Include descendant processes (children first).

**--dry-run**  
Show kill targets without sending signals.

**--json**  
Emit JSON on **ports**, **clean**, **project**, and **history** (colors disabled).

**-h**, **--help**  
Print help.

## TUI KEYS

| Key | Action |
| --- | --- |
| ↑/↓, j | Move selection |
| g / G | First / last row |
| PgUp/PgDn, Ctrl-u/d | Page |
| / | Search |
| p | Ports-only filter |
| k / K | SIGTERM / SIGKILL preview; y confirms |
| t / T | Tree kill preview |
| P | Project view |
| e | Tree view |
| i / Enter | Detail panel |
| r | Refresh |
| q | Quit |

## FILES

Linux history: `~/.local/share/com.sweeper.sweeper/history.json`  
macOS history: `~/Library/Application Support/com.sweeper.sweeper/history.json`  
Protect config: `~/.config/sweeper/protect.toml` (Linux) or Application Support path (macOS).

## SEE ALSO

README.md in the repository for platform compatibility and install instructions.
