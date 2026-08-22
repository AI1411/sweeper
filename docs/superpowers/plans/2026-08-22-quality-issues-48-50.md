# Quality Issues #48–#50 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship Sweeper quality issues #48 (kill summary + ports released), #49 (human-readable clean reasons), and #50 (TUI nav + kill preview) as three separate PRs merged in order.

**Architecture:** Extend `src/report.rs` with shared kill summary helpers; extend `src/clean/mod.rs` for formatted reasons; extend `src/tui/app.rs` for navigation and preview strings. One PR per GitHub issue.

**Tech Stack:** Rust, ratatui 0.29, existing `style` / `ProcessInfo` types.

---

See issue definitions: `docs/superpowers/issues/18-*.md`, `19-*.md`, `20-*.md` (GitHub #48, #49, #50).
