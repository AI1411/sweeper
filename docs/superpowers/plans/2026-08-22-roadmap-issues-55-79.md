# Roadmap Issues #55–#79 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement GitHub issues #55–#79 sequentially with one PR per issue merged to `main`.

**Architecture:** One issue → one branch `cursor/issue-NN-*-a446` → PR → CI → merge. Reuse shared modules (`process/plan.rs`, `report.rs`, `clean/mod.rs`, `tui/app.rs`).

**Tech Stack:** Rust, clap, ratatui, sysinfo, nix, lsof (ports MVP).

## Global Constraints

- Branch suffix `-a446`, prefix `cursor/`
- `cargo test --all-targets`, `cargo fmt --check`, `clippy -D warnings`
- No auto-kill; SIGTERM default
- Commit message includes `Fixes #NN`

---

## Progress

| Issue | Title | PR | Status |
|-------|-------|-----|--------|
| #55 | TUI kill confirmation | #81 | merged |
| #56 | Protect list + config | #82 | merged |
| #57 | Clean confidence | #83 | merged |
| #58 | --dry-run | #84 | merged |
| #59 | TUI detail panel | pending | in PR |
| #60–#79 | See issues README | — | pending |

See `docs/superpowers/issues/README.md` for specs 21–45.
