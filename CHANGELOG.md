# Changelog

All notable changes to Sweeper are documented in this file.

## Unreleased

- CLI kill flows share an inspect-then-select loop: numbered rows show port and project, then `all` / `1-3` / `high` / `q`.
- `sw node` merges listening ports into the match list so you can tell which Node is which.
- TUI: type to search without `/` first; PID digits match; help footer stays visible when the detail panel is closed; footer no longer doubles brackets.

## 0.1.0 — 2026-08-23

Initial public MVP and post-MVP feature set:

- CLI/TUI process browser (`sw`)
- Name and port targeting, project grouping, clean proposals
- Native port resolution on Linux and macOS with `lsof` fallback
- Kill history, protect list, dry-run, JSON output, and CI quality gates
