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
sw ports     # list LISTEN ports
sw top       # CPU / memory leaders
sw clean     # propose leftovers (you confirm)
sw history   # kill history
sw history --last
sw :3000 --force
```

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
