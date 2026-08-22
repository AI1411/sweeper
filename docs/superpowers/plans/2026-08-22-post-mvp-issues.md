# Post-MVP Issues Implementation Plan

> **For agentic workers:** Execute issues **one at a time**. For each issue: implement → test → commit → push → open PR → wait for CI → merge → close issue → start the next.

**Goal:** Ship GitHub issues #33–#38 in order.

**Architecture:** Keep shared logic in `process` / new small modules (`project`, tree helpers). CLI commands and TUI call the same core. No tokio; sync I/O.

**Tech stack:** Existing Rust crate (`clap`, `ratatui`, `sysinfo`, `nix`, `owo-colors`).

---

## Issue order

| # | Issue | Branch suffix |
|---|---|---|
| 33 | `sw project` group/kill | `cursor/issue-33-project-3a23` |
| 34 | `--tree` process tree kill | `cursor/issue-34-tree-kill-3a23` |
| 35 | TUI listening ports | `cursor/issue-35-tui-ports-3a23` |
| 36 | Multi-port kill UX | `cursor/issue-36-multi-port-3a23` |
| 37 | `sw clean` reasons + filters | `cursor/issue-37-clean-reasons-3a23` |
| 38 | Estimated memory freed | `cursor/issue-38-memory-freed-3a23` |

---

### Issue 33: `sw project`

**Files:** `src/project/mod.rs`, `src/commands/project.rs`, `src/main.rs`, `src/lib.rs`, `tests/project_group.rs`, `README.md`

- Infer project from `cwd` basename (primary) or command path heuristics
- `sw project` lists groups (name, path, count, memory, ports)
- `sw project <name>` shows members; confirm → kill each (respect protect)
- Unit tests with fixture `ProcessInfo`

### Issue 34: `--tree`

**Files:** `src/process/tree.rs`, wire `name`/`port`/`tui`, tests

- Collect descendants via PPID BFS/DFS from snapshot
- CLI `--tree` expands kill set; TUI `k`/`K` prompt or second key for tree (prefer: `t` kill tree / keep `k` single — or modal). Spec: offer Kill process vs Kill process tree — use `k` = process, `T` = tree (document in help) to avoid blocking modal in raw mode. Or: `k` asks if selection has children. Simplest solid UX: `k`/`K` kill selection only; `t`/`T` kill selection+descendants.

### Issue 35: TUI ports

**Files:** `src/tui/{mod,app,ui}.rs`

- Ensure async port merge updates table; status when loaded
- Search matches port numbers
- Optional: filter key for “has port” if cheap

### Issue 36: Multi-port UX

**Files:** `src/commands/port.rs`, tests for PID dedupe

- Build unique PID table across ports; one summary; one confirm for all

### Issue 37: Clean reasons

**Files:** `src/clean/mod.rs`, `src/commands/clean.rs`, tests

- Tag reasons; `--exclude <substr>` flag (or env `SWEEPER_CLEAN_EXCLUDE`)

### Issue 38: Memory freed

**Files:** shared helper + name/port/clean/tui

- Sum `memory_bytes` for successful kills only; print estimate

---

## Per-issue checklist

1. Branch from updated `main`
2. TDD where pure logic exists
3. `cargo test` + `cargo fmt` + `clippy -D warnings`
4. PR → CI green → merge → `gh issue close <n> --comment "Fixed in …"`
5. Proceed to next issue
