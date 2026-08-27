# Improvement Backlog (#137–#151) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement GitHub issues #137–#151 sequentially — one issue, one PR, merge, next.

**Architecture:** One branch per issue (`cursor/issue-NNN-*-e84c`). Minimal focused diffs per issue. TDD where pure logic allows.

**Tech Stack:** Rust (clap, ratatui, sysinfo, nix)

## Global Constraints

- Branch names: `cursor/<descriptive-name>-e84c`
- One issue → one PR → merge → next issue
- `cargo test`, `cargo fmt --check`, `cargo clippy -- -D warnings` before each PR
- No auto-kill behavior changes; confirmation flows unchanged (no `-y`)
- Safety: SIGTERM default; protected processes skipped

---

## Issue order

| # | GitHub | Branch suffix |
|---|--------|---------------|
| 137 | Clean CLI batch confirm | `issue-137-clean-batch-e84c` |
| 138 | Project git/compose detection | `issue-138-project-git-e84c` |
| 139 | History enhancements | `issue-139-history-e84c` |
| 140 | TUI context help | `issue-140-tui-help-e84c` |
| 141 | TUI auto-refresh | `issue-141-tui-refresh-e84c` |
| 142 | TUI sort toggle | `issue-142-tui-sort-e84c` |
| 143 | Fuzzy search | `issue-143-fuzzy-search-e84c` |
| 144 | User config | `issue-144-config-e84c` |
| 145 | JSON top | `issue-145-json-top-e84c` |
| 146 | System reuse | `issue-146-system-reuse-e84c` |
| 147 | Port batch cache | `issue-147-port-batch-e84c` |
| 148 | merge_ports HashMap | `issue-148-merge-ports-e84c` |
| 149 | TUI conditional redraw | `issue-149-tui-redraw-e84c` |
| 150 | Parallel SIGTERM | `issue-150-kill-batch-e84c` |
| 151 | Cache dir size | `issue-151-cache-size-e84c` |

See individual issue files in `docs/superpowers/issues/56-70-*.md` for acceptance criteria.
