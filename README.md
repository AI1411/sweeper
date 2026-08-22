# sweeper

Sweep unwanted processes away.

Developer-focused CLI/TUI for finding and safely terminating leftover processes on macOS (Linux works for most commands).

## Install (dev)

```bash
cargo install --path .
```

Binary name: `sw`

## Usage

```bash
sw           # TUI process browser
sw node      # find by name
sw :3000     # find by port
sw ports     # list LISTEN ports (alias: p)
sw top       # CPU / memory leaders (alias: t)
sw clean     # propose leftovers (you confirm) (alias: c)
sw history   # kill history (alias: h)
sw history --last
sw :3000 --force
```

Short aliases: `sw p`, `sw t`, `sw c`, `sw h`, `sw proj`.

### TUI keys

| Key | Action |
|---|---|
| ↑ / ↓ | Move |
| Space | Select / deselect |
| `/` | Search |
| `k` | SIGTERM |
| `K` | SIGKILL |
| `r` | Refresh |
| `q` | Quit |

## Safety

- Default signal is SIGTERM; SIGKILL only with `--force` or TUI `K`
- Critical macOS process names are protected
- `sw clean` never auto-kills — it proposes, you decide
