---
title: "[Config] User config file (config.toml)"
labels: [enhancement, post-mvp]
depends_on: []
priority: medium
---

## Summary

Add `~/.config/sweeper/config.toml` (XDG) for user preferences beyond `protect.toml`.

## Motivation

Today only `protect.toml` and env vars (`SWEEPER_CLEAN_EXCLUDE`) configure behavior. Common preferences (default TUI view, clean excludes, refresh interval) deserve a single config file.

## Acceptance criteria

- [ ] Load config from XDG / macOS Application Support (same pattern as protect)
- [ ] Supported keys (v1):
  - `clean.exclude = ["postgres", "redis"]`
  - `clean.default_high_only = true`
  - `tui.auto_refresh_secs = 3`
  - `tui.default_view = "processes" | "projects"`
- [ ] Env vars override config file values
- [ ] `sw doctor` reports config path and parse errors
- [ ] Invalid TOML → warn once, fall back to defaults
- [ ] Tests with temp config files

## Example

```toml
[clean]
exclude = ["postgres", "redis"]
default_high_only = true

[tui]
auto_refresh_secs = 3
```

## Implementation notes

- New `src/config/mod.rs`
- Wire into `clean`, `tui`, `doctor`

## References

- `src/process/protect.rs` (config path pattern)
- `src/commands/doctor.rs`
