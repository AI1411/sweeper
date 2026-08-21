# Sweeper MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the Sweeper MVP CLI/TUI (`sw`) on macOS: process list with search and multi-select kill, name search, port search, `top`, and `clean` proposals.

**Architecture:** Single Rust binary. `clap` resolves targets; shared `process` core lists processes (sysinfo), resolves ports (lsof), and kills (nix). CLI commands and a ratatui TUI both call that core. History is JSON under Application Support.

**Tech Stack:** Rust, clap, ratatui, crossterm, sysinfo, nix, serde/serde_json, directories, time, thiserror, anyhow

## Global Constraints

- Platform: macOS first (Linux out of scope for this plan)
- Binary name: `sw` (`[[bin]] name = "sw"`)
- Default signal: SIGTERM; SIGKILL only via `--force` or TUI `K`
- Never auto-kill from `clean` — propose only; user decides
- No `-y` / `--yes` in MVP
- No tokio — use sync I/O and `std::thread` where needed
- Spec: `docs/superpowers/specs/2026-08-21-tech-stack-design.md`
- Requirements: `docs/requirements.md` (§22 MVP)

## File Structure

```text
Cargo.toml
src/
  main.rs                 # binary entry → anyhow, dispatch
  lib.rs                  # re-exports modules for tests
  error.rs                # thiserror types
  cli.rs                  # clap + Target resolution
  process/
    mod.rs
    types.rs              # ProcessInfo
    list.rs               # sysinfo snapshot
    ports.rs              # lsof parse + merge
    kill.rs               # SIGTERM/SIGKILL + wait
    protect.rs            # protected name list
  history/
    mod.rs                # JSON load/save/append
  clean/
    mod.rs                # leftover heuristics
  commands/
    mod.rs
    name.rs               # sw <name>
    port.rs               # sw :3000
    top.rs                # sw top
    clean.rs              # sw clean
    history.rs            # sw history
  tui/
    mod.rs                # run loop
    app.rs                # state + key handling
    ui.rs                 # ratatui widgets
tests/
  cli_target.rs
  protect.rs
  history_store.rs
  ports_parse.rs
```

---

### Task 1: Cargo scaffold + error types + ProcessInfo

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`
- Create: `src/error.rs`
- Create: `src/process/mod.rs`
- Create: `src/process/types.rs`
- Test: `tests/process_info.rs` (optional smoke — prefer unit tests in types if needed)

**Interfaces:**
- Produces: `ProcessInfo { pid: u32, ppid: u32, name: String, cpu: f32, memory_bytes: u64, ports: Vec<u16>, command: Option<String>, cwd: Option<String> }`
- Produces: `SweeperError` / `Result<T>` alias

- [ ] **Step 1: Create Cargo project with dependencies**

```bash
cd /workspace
cargo init --name sweeper
```

Edit `Cargo.toml` to:

```toml
[package]
name = "sweeper"
version = "0.1.0"
edition = "2021"
description = "Sweep unwanted processes away"
license = "MIT"

[[bin]]
name = "sw"
path = "src/main.rs"

[dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
crossterm = "0.28"
directories = "5"
nix = { version = "0.29", features = ["signal", "process"] }
ratatui = "0.29"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sysinfo = "0.33"
thiserror = "2"
time = { version = "0.3", features = ["formatting", "local-offset", "serde"] }

[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 2: Write `src/error.rs`**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SweeperError {
    #[error("process not found: {0}")]
    ProcessNotFound(String),
    #[error("port not in use: {0}")]
    PortNotInUse(u16),
    #[error("protected process: {0} (pid {1})")]
    Protected(String, u32),
    #[error("lsof failed: {0}")]
    Lsof(String),
    #[error("kill failed for pid {0}: {1}")]
    Kill(u32, String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, SweeperError>;
```

- [ ] **Step 3: Write `src/process/types.rs` and `mod.rs`**

```rust
// src/process/types.rs
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
    pub cpu: f32,
    pub memory_bytes: u64,
    pub ports: Vec<u16>,
    pub command: Option<String>,
    pub cwd: Option<String>,
}

impl ProcessInfo {
    pub fn memory_mb(&self) -> f64 {
        self.memory_bytes as f64 / (1024.0 * 1024.0)
    }
}
```

```rust
// src/process/mod.rs
pub mod types;

pub use types::ProcessInfo;
```

- [ ] **Step 4: Wire `lib.rs` and stub `main.rs`**

```rust
// src/lib.rs
pub mod error;
pub mod process;

pub use error::{Result, SweeperError};
pub use process::ProcessInfo;
```

```rust
// src/main.rs
fn main() -> anyhow::Result<()> {
    println!("sweeper scaffold");
    Ok(())
}
```

- [ ] **Step 5: Verify build**

Run: `cargo build`
Expected: success, binary at `target/debug/sw`

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src/
git commit -m "chore: scaffold sweeper Rust project with ProcessInfo"
```

---

### Task 2: CLI target resolution

**Files:**
- Create: `src/cli.rs`
- Modify: `src/lib.rs`
- Modify: `src/main.rs`
- Test: `tests/cli_target.rs`

**Interfaces:**
- Produces: `enum Target { Tui, Name(String), Ports(Vec<u16>), Sub(SubCommand) }`
- Produces: `enum SubCommand { Ports, Top, Clean, History { last: bool }, Project { name: Option<String> } }`
- Produces: `struct Cli { force: bool, tree: bool, target: Target }` via `Cli::parse` / `Cli::parse_from`

- [ ] **Step 1: Write failing tests**

```rust
// tests/cli_target.rs
use sweeper::cli::{Cli, SubCommand, Target};

#[test]
fn bare_sw_is_tui() {
    let cli = Cli::parse_from(["sw"]);
    assert_eq!(cli.target, Target::Tui);
    assert!(!cli.force);
}

#[test]
fn name_search() {
    let cli = Cli::parse_from(["sw", "node"]);
    assert_eq!(cli.target, Target::Name("node".into()));
}

#[test]
fn single_port() {
    let cli = Cli::parse_from(["sw", ":3000"]);
    assert_eq!(cli.target, Target::Ports(vec![3000]));
}

#[test]
fn multiple_ports() {
    let cli = Cli::parse_from(["sw", ":3000", ":3001"]);
    assert_eq!(cli.target, Target::Ports(vec![3000, 3001]));
}

#[test]
fn subcommand_top() {
    let cli = Cli::parse_from(["sw", "top"]);
    assert_eq!(cli.target, Target::Sub(SubCommand::Top));
}

#[test]
fn subcommand_history_last() {
    let cli = Cli::parse_from(["sw", "history", "--last"]);
    assert_eq!(cli.target, Target::Sub(SubCommand::History { last: true }));
}

#[test]
fn force_flag() {
    let cli = Cli::parse_from(["sw", ":3000", "--force"]);
    assert!(cli.force);
    assert_eq!(cli.target, Target::Ports(vec![3000]));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test cli_target`
Expected: FAIL (module `cli` missing)

- [ ] **Step 3: Implement `src/cli.rs`**

```rust
use clap::{Parser, Subcommand as ClapSubcommand};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Tui,
    Name(String),
    Ports(Vec<u16>),
    Sub(SubCommand),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubCommand {
    Ports,
    Top,
    Clean,
    History { last: bool },
    Project { name: Option<String> },
}

#[derive(Debug, Parser)]
#[command(name = "sw", about = "Sweep unwanted processes away")]
pub struct CliArgs {
    /// Use SIGKILL when needed / requested
    #[arg(long, global = true)]
    pub force: bool,

    /// Kill process tree (MVP: accepted, behavior may stub)
    #[arg(long, global = true)]
    pub tree: bool,

    #[command(subcommand)]
    subcommand: Option<RawSub>,

    /// Positional targets: names and/or :ports
    #[arg(trailing_var_arg = true, allow_hyphen_values = false)]
    raw_targets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cli {
    pub force: bool,
    pub tree: bool,
    pub target: Target,
}

#[derive(Debug, ClapSubcommand)]
enum RawSub {
    Ports,
    Top,
    Clean,
    History {
        #[arg(long)]
        last: bool,
    },
    Project {
        name: Option<String>,
    },
}

impl Cli {
    pub fn parse_from<I, T>(itr: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        let args = CliArgs::parse_from(itr);
        Self::from_args(args)
    }

    pub fn parse() -> Self {
        Self::from_args(CliArgs::parse())
    }

    fn from_args(args: CliArgs) -> Self {
        let target = resolve_target(args.subcommand, args.raw_targets);
        Self {
            force: args.force,
            tree: args.tree,
            target,
        }
    }
}

fn resolve_target(subcommand: Option<RawSub>, raw_targets: Vec<String>) -> Target {
    if let Some(sub) = subcommand {
        return match sub {
            RawSub::Ports => Target::Sub(SubCommand::Ports),
            RawSub::Top => Target::Sub(SubCommand::Top),
            RawSub::Clean => Target::Sub(SubCommand::Clean),
            RawSub::History { last } => Target::Sub(SubCommand::History { last }),
            RawSub::Project { name } => Target::Sub(SubCommand::Project { name }),
        };
    }

    if raw_targets.is_empty() {
        return Target::Tui;
    }

    let mut ports = Vec::new();
    let mut names = Vec::new();
    for t in raw_targets {
        if let Some(p) = t.strip_prefix(':') {
            if let Ok(port) = p.parse::<u16>() {
                ports.push(port);
                continue;
            }
        }
        names.push(t);
    }

    if !ports.is_empty() && names.is_empty() {
        return Target::Ports(ports);
    }
    if ports.is_empty() && names.len() == 1 {
        return Target::Name(names.remove(0));
    }
    // Mixed or multiple names: treat first as name (MVP)
    Target::Name(names.into_iter().next().unwrap_or_default())
}
```

Add to `lib.rs`: `pub mod cli;`

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test cli_target`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/cli.rs src/lib.rs tests/cli_target.rs
git commit -m "feat: resolve sw CLI targets (name, port, subcommands)"
```

---

### Task 3: Protected process list

**Files:**
- Create: `src/process/protect.rs`
- Modify: `src/process/mod.rs`
- Test: `tests/protect.rs`

**Interfaces:**
- Produces: `fn is_protected(name: &str) -> bool`
- Consumes: process name string (basename)

- [ ] **Step 1: Write failing tests**

```rust
// tests/protect.rs
use sweeper::process::protect::is_protected;

#[test]
fn protects_system_daemons() {
    assert!(is_protected("kernel_task"));
    assert!(is_protected("launchd"));
    assert!(is_protected("WindowServer"));
}

#[test]
fn allows_dev_processes() {
    assert!(!is_protected("node"));
    assert!(!is_protected("vite"));
}
```

- [ ] **Step 2: Run test — expect FAIL**

Run: `cargo test --test protect`
Expected: FAIL

- [ ] **Step 3: Implement**

```rust
// src/process/protect.rs
const PROTECTED: &[&str] = &[
    "kernel_task",
    "launchd",
    "WindowServer",
    "loginwindow",
    "SystemUIServer",
    "Finder",
    "Dock",
];

pub fn is_protected(name: &str) -> bool {
    let base = name.rsplit('/').next().unwrap_or(name);
    PROTECTED.iter().any(|p| base.eq_ignore_ascii_case(p))
}
```

Export in `process/mod.rs`: `pub mod protect;`

- [ ] **Step 4: Run tests — expect PASS**

Run: `cargo test --test protect`

- [ ] **Step 5: Commit**

```bash
git add src/process/protect.rs src/process/mod.rs tests/protect.rs
git commit -m "feat: protect critical macOS process names from kill"
```

---

### Task 4: Process listing via sysinfo

**Files:**
- Create: `src/process/list.rs`
- Modify: `src/process/mod.rs`

**Interfaces:**
- Produces: `fn list_processes() -> Vec<ProcessInfo>`
- Produces: `fn find_by_name_fuzzy(query: &str) -> Vec<ProcessInfo>`
- Consumes: `ProcessInfo`, sysinfo

- [ ] **Step 1: Implement `list.rs`**

```rust
use sysinfo::{ProcessesToUpdate, System};

use super::types::ProcessInfo;

pub fn list_processes() -> Vec<ProcessInfo> {
    let mut sys = System::new_all();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let mut out = Vec::new();
    for (pid, proc_) in sys.processes() {
        let name = proc_.name().to_string_lossy().into_owned();
        let cmd = {
            let args: Vec<String> = proc_
                .cmd()
                .iter()
                .map(|s| s.to_string_lossy().into_owned())
                .collect();
            if args.is_empty() {
                None
            } else {
                Some(args.join(" "))
            }
        };
        let cwd = proc_
            .cwd()
            .map(|p| p.to_string_lossy().into_owned());

        out.push(ProcessInfo {
            pid: pid.as_u32(),
            ppid: proc_.parent().map(|p| p.as_u32()).unwrap_or(0),
            name,
            cpu: proc_.cpu_usage(),
            memory_bytes: proc_.memory(),
            ports: Vec::new(),
            command: cmd,
            cwd,
        });
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

pub fn find_by_name_fuzzy(query: &str) -> Vec<ProcessInfo> {
    let q = query.to_lowercase();
    list_processes()
        .into_iter()
        .filter(|p| {
            p.name.to_lowercase().contains(&q)
                || p.command
                    .as_ref()
                    .map(|c| c.to_lowercase().contains(&q))
                    .unwrap_or(false)
        })
        .collect()
}
```

Note: Adjust `sysinfo` 0.33 API if method names differ after `cargo check` — fix to compile against the locked version.

- [ ] **Step 2: Manual smoke**

```bash
cargo run --quiet -- # still stub main
# temporary: add a dbg in a #[cfg(test)] or `examples/` if needed
cargo test
```

Add a unit test that only checks fuzzy filter logic with constructed `ProcessInfo` if sysinfo in CI is awkward — prefer:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_matches_substring() {
        // integration-style: just ensure list_processes returns something on macOS/Linux agents
        let all = list_processes();
        assert!(!all.is_empty());
    }
}
```

- [ ] **Step 3: Commit**

```bash
git add src/process/list.rs src/process/mod.rs
git commit -m "feat: list processes with sysinfo and fuzzy name filter"
```

---

### Task 5: Port resolution via lsof

**Files:**
- Create: `src/process/ports.rs`
- Modify: `src/process/mod.rs`
- Test: `tests/ports_parse.rs`

**Interfaces:**
- Produces: `fn parse_lsof_listen_line(line: &str) -> Option<(u32, u16)>`
- Produces: `fn listening_ports() -> Result<Vec<(u16, u32)>>`  // port, pid
- Produces: `fn pids_for_port(port: u16) -> Result<Vec<u32>>`
- Produces: `fn merge_ports(procs: &mut [ProcessInfo], port_map: &[(u16, u32)])`

- [ ] **Step 1: Write failing parse tests**

```rust
// tests/ports_parse.rs
use sweeper::process::ports::parse_lsof_listen_line;

#[test]
fn parses_typical_lsof_line() {
    // COMMAND PID USER FD TYPE DEVICE SIZE/OFF NODE NAME
    let line = "node 48291 user 20u IPv4 0x0 0t0 TCP *:3000 (LISTEN)";
    let (pid, port) = parse_lsof_listen_line(line).expect("parse");
    assert_eq!(pid, 48291);
    assert_eq!(port, 3000);
}

#[test]
fn ignores_non_listen() {
    let line = "node 48291 user 20u IPv4 0x0 0t0 TCP 127.0.0.1:3000->127.0.0.1:4000 (ESTABLISHED)";
    assert!(parse_lsof_listen_line(line).is_none());
}
```

- [ ] **Step 2: Run — expect FAIL**

Run: `cargo test --test ports_parse`

- [ ] **Step 3: Implement `ports.rs`**

```rust
use std::process::Command;

use crate::error::{Result, SweeperError};
use crate::process::ProcessInfo;

/// Parse one `lsof -nP -iTCP -sTCP:LISTEN` style line → (pid, port)
pub fn parse_lsof_listen_line(line: &str) -> Option<(u32, u16)> {
    if !line.contains("(LISTEN)") {
        return None;
    }
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 9 {
        return None;
    }
    let pid: u32 = parts[1].parse().ok()?;
    let name_field = parts.last()?; // e.g. *:3000 or 127.0.0.1:3000
    let port_str = name_field.rsplit(':').next()?;
    let port_str = port_str.trim_end_matches("(LISTEN)");
    // last field is often "*:3000" and "(LISTEN)" is separate — handle both
    let port_str = if parts.contains(&"(LISTEN)") {
        parts
            .iter()
            .rev()
            .find(|p| p.contains(':') && !p.contains("->"))?
            .rsplit(':')
            .next()?
    } else {
        port_str
    };
    let port: u16 = port_str.parse().ok()?;
    Some((pid, port))
}

pub fn listening_ports() -> Result<Vec<(u16, u32)>> {
    let output = Command::new("lsof")
        .args(["-nP", "-iTCP", "-sTCP:LISTEN"])
        .output()
        .map_err(|e| SweeperError::Lsof(e.to_string()))?;
    if !output.status.success() && output.stdout.is_empty() {
        return Err(SweeperError::Lsof(format!(
            "exit {}",
            output.status.code().unwrap_or(-1)
        )));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut pairs = Vec::new();
    for line in text.lines().skip(1) {
        if let Some((pid, port)) = parse_lsof_listen_line(line) {
            pairs.push((port, pid));
        }
    }
    Ok(pairs)
}

pub fn pids_for_port(port: u16) -> Result<Vec<u32>> {
    let output = Command::new("lsof")
        .args([
            "-nP",
            &format!("-iTCP:{}", port),
            "-sTCP:LISTEN",
            "-t",
        ])
        .output()
        .map_err(|e| SweeperError::Lsof(e.to_string()))?;
    let text = String::from_utf8_lossy(&output.stdout);
    let mut pids = Vec::new();
    for line in text.lines() {
        if let Ok(pid) = line.trim().parse::<u32>() {
            pids.push(pid);
        }
    }
    Ok(pids)
}

pub fn merge_ports(procs: &mut [ProcessInfo], port_map: &[(u16, u32)]) {
    for (port, pid) in port_map {
        if let Some(p) = procs.iter_mut().find(|p| p.pid == *pid) {
            if !p.ports.contains(port) {
                p.ports.push(*port);
            }
        }
    }
}
```

Fix parse logic carefully so tests pass against the sample lines.

- [ ] **Step 4: Run tests — PASS**

Run: `cargo test --test ports_parse`

- [ ] **Step 5: Commit**

```bash
git add src/process/ports.rs src/process/mod.rs tests/ports_parse.rs
git commit -m "feat: resolve LISTEN ports via lsof parsing"
```

---

### Task 6: Kill flow (SIGTERM → wait → optional SIGKILL)

**Files:**
- Create: `src/process/kill.rs`
- Modify: `src/process/mod.rs`

**Interfaces:**
- Produces: `pub enum KillOutcome { Terminated, ForceKilled, StillAlive, SkippedProtected }`
- Produces: `fn kill_pid(pid: u32, name: &str, force: bool) -> Result<KillOutcome>`
- Consumes: `protect::is_protected`, nix signal

- [ ] **Step 1: Implement kill helper**

```rust
use std::thread;
use std::time::Duration;

use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

use crate::error::{Result, SweeperError};
use crate::process::protect::is_protected;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KillOutcome {
    Terminated,
    ForceKilled,
    StillAlive,
    SkippedProtected,
}

fn pid_alive(pid: u32) -> bool {
    // signal 0 checks existence
    kill(Pid::from_raw(pid as i32), None).is_ok()
}

pub fn kill_pid(pid: u32, name: &str, force: bool) -> Result<KillOutcome> {
    if is_protected(name) {
        return Ok(KillOutcome::SkippedProtected);
    }

    let raw = Pid::from_raw(pid as i32);
    kill(raw, Signal::SIGTERM).map_err(|e| SweeperError::Kill(pid, e.to_string()))?;

    thread::sleep(Duration::from_secs(2));

    if !pid_alive(pid) {
        return Ok(KillOutcome::Terminated);
    }

    if force {
        kill(raw, Signal::SIGKILL).map_err(|e| SweeperError::Kill(pid, e.to_string()))?;
        thread::sleep(Duration::from_millis(200));
        if !pid_alive(pid) {
            return Ok(KillOutcome::ForceKilled);
        }
        return Ok(KillOutcome::StillAlive);
    }

    Ok(KillOutcome::StillAlive)
}
```

- [ ] **Step 2: `cargo check` and fix nix `kill(..., None)` API**

Use the correct nix 0.29 signature for existence check (`Signal` optional or `libc::kill(pid, 0)`).

- [ ] **Step 3: Commit**

```bash
git add src/process/kill.rs src/process/mod.rs
git commit -m "feat: SIGTERM-first kill with optional SIGKILL"
```

---

### Task 7: History store

**Files:**
- Create: `src/history/mod.rs`
- Modify: `src/lib.rs`
- Test: `tests/history_store.rs`

**Interfaces:**
- Produces: `struct HistoryEntry { time, pid, name, ports, signal, result }`
- Produces: `fn history_path() -> PathBuf`
- Produces: `fn append_entry(entry: HistoryEntry) -> Result<()>` (cap 200)
- Produces: `fn load_entries() -> Result<Vec<HistoryEntry>>`
- Produces: `fn last_entry() -> Result<Option<HistoryEntry>>`

- [ ] **Step 1: Write failing tests with tempfile**

```rust
// tests/history_store.rs
use sweeper::history::{append_entry_at, load_entries_at, HistoryEntry, KillSignal};
use tempfile::tempdir;

#[test]
fn append_and_load() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("history.json");
    let e = HistoryEntry::new(123, "node", vec![3000], KillSignal::Term, "terminated");
    append_entry_at(&path, e.clone()).unwrap();
    let all = load_entries_at(&path).unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].pid, 123);
}
```

- [ ] **Step 2: Implement history module with injectable path for tests**

```rust
use std::fs;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::error::Result;

const MAX_ENTRIES: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum KillSignal {
    Term,
    Kill,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HistoryEntry {
    pub time: OffsetDateTime,
    pub pid: u32,
    pub name: String,
    pub ports: Vec<u16>,
    pub signal: KillSignal,
    pub result: String,
}

impl HistoryEntry {
    pub fn new(
        pid: u32,
        name: impl Into<String>,
        ports: Vec<u16>,
        signal: KillSignal,
        result: impl Into<String>,
    ) -> Self {
        Self {
            time: OffsetDateTime::now_utc(),
            pid,
            name: name.into(),
            ports,
            signal,
            result: result.into(),
        }
    }
}

pub fn history_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("com", "sweeper", "sweeper")
        .expect("home directory");
    let dir = dirs.data_dir();
    fs::create_dir_all(dir)?;
    Ok(dir.join("history.json"))
}

pub fn load_entries_at(path: &Path) -> Result<Vec<HistoryEntry>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(path)?;
    if data.trim().is_empty() {
        return Ok(Vec::new());
    }
    Ok(serde_json::from_str(&data)?)
}

pub fn append_entry_at(path: &Path, entry: HistoryEntry) -> Result<()> {
    let mut entries = load_entries_at(path)?;
    entries.push(entry);
    if entries.len() > MAX_ENTRIES {
        let skip = entries.len() - MAX_ENTRIES;
        entries = entries.into_iter().skip(skip).collect();
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(&entries)?)?;
    Ok(())
}

pub fn append_entry(entry: HistoryEntry) -> Result<()> {
    append_entry_at(&history_path()?, entry)
}

pub fn load_entries() -> Result<Vec<HistoryEntry>> {
    load_entries_at(&history_path()?)
}

pub fn last_entry() -> Result<Option<HistoryEntry>> {
    Ok(load_entries()?.into_iter().next_back())
}
```

Enable `time` serde features as needed (`serde-human-readable` or timestamp). If OffsetDateTime serde is painful, store `time: String` RFC3339 instead.

- [ ] **Step 3: Tests PASS**

Run: `cargo test --test history_store`

- [ ] **Step 4: Commit**

```bash
git add src/history/mod.rs src/lib.rs tests/history_store.rs Cargo.toml
git commit -m "feat: persist kill history as capped JSON"
```

---

### Task 8: Interactive confirm helper + name/port/top commands

**Files:**
- Create: `src/commands/mod.rs`
- Create: `src/commands/confirm.rs`
- Create: `src/commands/name.rs`
- Create: `src/commands/port.rs`
- Create: `src/commands/top.rs`
- Modify: `src/main.rs`, `src/lib.rs`

**Interfaces:**
- Produces: `fn confirm(prompt: &str) -> bool` reading stdin `y/N`
- Produces: `fn run_name(query: &str, force: bool) -> anyhow::Result<()>`
- Produces: `fn run_ports(ports: &[u16], force: bool) -> anyhow::Result<()>`
- Produces: `fn run_top() -> anyhow::Result<()>`
- Consumes: list/fuzzy, pids_for_port, kill_pid, history::append_entry

- [ ] **Step 1: Implement confirm**

```rust
// src/commands/confirm.rs
use std::io::{self, Write};

pub fn confirm(prompt: &str) -> io::Result<bool> {
    print!("{prompt} [y/N] ");
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(matches!(buf.trim(), "y" | "Y" | "yes" | "YES"))
}
```

- [ ] **Step 2: Implement name command**

```rust
// src/commands/name.rs
use crate::history::{append_entry, HistoryEntry, KillSignal};
use crate::process::kill::{kill_pid, KillOutcome};
use crate::process::list::find_by_name_fuzzy;
use super::confirm::confirm;

pub fn run_name(query: &str, force: bool) -> anyhow::Result<()> {
    let matches = find_by_name_fuzzy(query);
    if matches.is_empty() {
        println!("No processes matching '{query}'");
        return Ok(());
    }
    println!("Found {} processes\n", matches.len());
    for p in &matches {
        println!("  {:>6}  {}  {:.1}%  {:.0} MB", p.pid, p.name, p.cpu, p.memory_mb());
    }
    let total: u64 = matches.iter().map(|p| p.memory_bytes).sum();
    println!("\nTotal memory: {:.1} GB", total as f64 / 1e9);
    if !confirm("Kill all?")? {
        println!("Cancelled.");
        return Ok(());
    }
    for p in matches {
        let mut use_force = force;
        let mut outcome = kill_pid(p.pid, &p.name, use_force)?;
        if outcome == KillOutcome::StillAlive && !use_force {
            if confirm(&format!("PID {} still alive. Force kill?", p.pid))? {
                use_force = true;
                outcome = kill_pid(p.pid, &p.name, true)?;
            }
        }
        let signal = if use_force && outcome == KillOutcome::ForceKilled {
            KillSignal::Kill
        } else {
            KillSignal::Term
        };
        let _ = append_entry(HistoryEntry::new(
            p.pid,
            &p.name,
            p.ports.clone(),
            signal,
            format!("{outcome:?}"),
        ));
        println!("{} pid {}: {:?}", p.name, p.pid, outcome);
    }
    Ok(())
}
```

- [ ] **Step 3: Implement port command**

```rust
// src/commands/port.rs
use crate::history::{append_entry, HistoryEntry, KillSignal};
use crate::process::kill::{kill_pid, KillOutcome};
use crate::process::list::list_processes;
use crate::process::ports::pids_for_port;
use super::confirm::confirm;

pub fn run_ports(ports: &[u16], force: bool) -> anyhow::Result<()> {
    let procs = list_processes();
    for port in ports {
        let pids = pids_for_port(*port)?;
        if pids.is_empty() {
            println!("PORT {port}: not in use");
            continue;
        }
        for pid in pids {
            let info = procs.iter().find(|p| p.pid == pid);
            let name = info.map(|p| p.name.as_str()).unwrap_or("?");
            let cpu = info.map(|p| p.cpu).unwrap_or(0.0);
            let mem = info.map(|p| p.memory_mb()).unwrap_or(0.0);
            println!("PORT  PID    PROCESS     CPU    MEM");
            println!("{port:<5} {pid:<6} {name:<10} {cpu:.1}%  {mem:.0}MB");
            if !confirm("Kill this process?")? {
                continue;
            }
            let mut use_force = force;
            let mut outcome = kill_pid(pid, name, use_force)?;
            if outcome == KillOutcome::StillAlive && !use_force {
                if confirm("Force kill?")? {
                    use_force = true;
                    outcome = kill_pid(pid, name, true)?;
                }
            }
            let signal = if matches!(outcome, KillOutcome::ForceKilled) {
                KillSignal::Kill
            } else {
                KillSignal::Term
            };
            let _ = append_entry(HistoryEntry::new(pid, name, vec![*port], signal, format!("{outcome:?}")));
            println!("{:?}", outcome);
        }
    }
    Ok(())
}
```

- [ ] **Step 4: Implement top**

```rust
// src/commands/top.rs
use crate::process::list::list_processes;

pub fn run_top() -> anyhow::Result<()> {
    let mut procs = list_processes();
    println!("CPU\n");
    let mut by_cpu = procs.clone();
    by_cpu.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap());
    for (i, p) in by_cpu.iter().take(10).enumerate() {
        println!("{}. {}  {:.0}%", i + 1, p.name, p.cpu);
    }
    println!("\nMEMORY\n");
    procs.sort_by_key(|p| std::cmp::Reverse(p.memory_bytes));
    for (i, p) in procs.iter().take(10).enumerate() {
        println!("{}. {}  {:.1} GB", i + 1, p.name, p.memory_bytes as f64 / 1e9);
    }
    Ok(())
}
```

MVP `top` is display-only; killing from top can be follow-up (TUI covers interactive kill).

- [ ] **Step 5: Wire `main.rs` dispatch for name/port/top**

```rust
use sweeper::cli::{Cli, SubCommand, Target};
use sweeper::commands::{name, port, top};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let force = cli.force;
    match cli.target {
        Target::Name(q) => name::run_name(&q, force)?,
        Target::Ports(ps) => port::run_ports(&ps, force)?,
        Target::Sub(SubCommand::Top) => top::run_top()?,
        Target::Tui => {
            println!("TUI not implemented yet");
        }
        other => {
            println!("Not implemented yet: {other:?}");
        }
    }
    Ok(())
}
```

- [ ] **Step 6: Manual verify**

```bash
cargo run -- top
cargo run -- :9999   # unlikely port — expect not in use
```

- [ ] **Step 7: Commit**

```bash
git add src/commands src/main.rs src/lib.rs
git commit -m "feat: add name, port, and top CLI commands"
```

---

### Task 9: `sw clean` + `sw history`

**Files:**
- Create: `src/clean/mod.rs`
- Create: `src/commands/clean.rs`
- Create: `src/commands/history.rs`
- Modify: `src/main.rs`, `src/lib.rs`, `src/commands/mod.rs`

**Interfaces:**
- Produces: `fn propose_leftovers(procs: &[ProcessInfo], ports: &[(u16, u32)]) -> Vec<ProcessInfo>`
- Produces: `fn run_clean(force: bool) -> anyhow::Result<()>`
- Produces: `fn run_history(last: bool) -> anyhow::Result<()>`

- [ ] **Step 1: Clean heuristics**

```rust
// src/clean/mod.rs
use crate::process::ProcessInfo;

const DEV_NAMES: &[&str] = &["node", "bun", "vite", "next-server", "python", "java"];

pub fn propose_leftovers(procs: &[ProcessInfo], listening: &[(u16, u32)]) -> Vec<ProcessInfo> {
    let listen_pids: std::collections::HashSet<u32> =
        listening.iter().map(|(_, pid)| *pid).collect();

    procs
        .iter()
        .filter(|p| {
            let name = p.name.to_lowercase();
            let is_dev = DEV_NAMES.iter().any(|d| name.contains(d));
            let orphan = p.ppid == 1 || p.ppid == 0;
            let has_listen = listen_pids.contains(&p.pid);
            is_dev && (orphan || has_listen)
        })
        .cloned()
        .collect()
}
```

- [ ] **Step 2: `run_clean` lists proposals, then confirm-and-kill each — no auto kill**

```rust
pub fn run_clean(force: bool) -> anyhow::Result<()> {
    use crate::clean::propose_leftovers;
    use crate::commands::confirm::confirm;
    use crate::history::{append_entry, HistoryEntry, KillSignal};
    use crate::process::kill::{kill_pid, KillOutcome};
    use crate::process::list::list_processes;
    use crate::process::ports::listening_ports;

    let mut procs = list_processes();
    let ports = listening_ports().unwrap_or_default();
    crate::process::ports::merge_ports(&mut procs, &ports);
    let proposals = propose_leftovers(&procs, &ports);
    println!("Sweeper found possible leftovers:\n");
    println!("✓ {} candidate processes", proposals.len());
    for p in &proposals {
        println!("  {} pid {} ports {:?}", p.name, p.pid, p.ports);
    }
    if proposals.is_empty() {
        return Ok(());
    }
    if !confirm("Select processes to clean (confirm each)?")? {
        println!("Cancelled.");
        return Ok(());
    }
    for p in proposals {
        if !confirm(&format!("Kill {} (pid {})?", p.name, p.pid))? {
            continue;
        }
        let mut use_force = force;
        let mut outcome = kill_pid(p.pid, &p.name, use_force)?;
        if outcome == KillOutcome::StillAlive && !use_force {
            if confirm("Force kill?")? {
                use_force = true;
                outcome = kill_pid(p.pid, &p.name, true)?;
            }
        }
        let signal = if matches!(outcome, KillOutcome::ForceKilled) {
            KillSignal::Kill
        } else {
            KillSignal::Term
        };
        let _ = append_entry(HistoryEntry::new(
            p.pid,
            &p.name,
            p.ports.clone(),
            signal,
            format!("{outcome:?}"),
        ));
        println!("{} pid {}: {:?}", p.name, p.pid, outcome);
    }
    Ok(())
}
```

- [ ] **Step 3: history command**

```rust
pub fn run_history(last: bool) -> anyhow::Result<()> {
    use crate::history::{last_entry, load_entries};
    if last {
        match last_entry()? {
            Some(e) => println!("{}  {}  PID {}  ports {:?}", e.time, e.name, e.pid, e.ports),
            None => println!("No history yet."),
        }
        return Ok(());
    }
    for e in load_entries()? {
        println!("{}  {}  PID {}  ports {:?}", e.time, e.name, e.pid, e.ports);
    }
    Ok(())
}
```

- [ ] **Step 4: Wire into main + commit**

```bash
git add src/clean src/commands src/main.rs src/lib.rs
git commit -m "feat: add clean proposals and history commands"
```

---

### Task 10: TUI (ratatui)

**Files:**
- Create: `src/tui/mod.rs`
- Create: `src/tui/app.rs`
- Create: `src/tui/ui.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Produces: `pub fn run() -> anyhow::Result<()>`
- Consumes: `list_processes`, `listening_ports`, `merge_ports`, `kill_pid`, `confirm` via in-TUI modal

- [ ] **Step 1: App state**

```rust
// src/tui/app.rs
use crate::process::ProcessInfo;

pub struct App {
    pub processes: Vec<ProcessInfo>,
    pub filtered: Vec<usize>, // indices into processes
    pub cursor: usize,
    pub selected: std::collections::HashSet<u32>, // pids
    pub query: String,
    pub searching: bool,
    pub should_quit: bool,
    pub status: String,
}

impl App {
    pub fn new(processes: Vec<ProcessInfo>) -> Self {
        let mut app = Self {
            processes,
            filtered: Vec::new(),
            cursor: 0,
            selected: Default::default(),
            query: String::new(),
            searching: false,
            should_quit: false,
            status: String::new(),
        };
        app.refilter();
        app
    }

    pub fn refilter(&mut self) {
        let q = self.query.to_lowercase();
        self.filtered = self
            .processes
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                q.is_empty()
                    || p.name.to_lowercase().contains(&q)
                    || p.ports.iter().any(|port| port.to_string().contains(&q))
            })
            .map(|(i, _)| i)
            .collect();
        if self.cursor >= self.filtered.len() && !self.filtered.is_empty() {
            self.cursor = self.filtered.len() - 1;
        }
    }
}
```

- [ ] **Step 2: UI draw**

Draw a table with columns PID / PROCESS / PORT / CPU / MEM, search line, help footer (`Space` Select `k` Kill `K` Force `q` Quit `/` Search). Mark selected rows with `*`.

- [ ] **Step 3: Event loop in `tui/mod.rs`**

- Enter raw mode, alternate screen
- On tick / key: update app
- `/` toggles searching; type filters; Esc ends search
- `Space` toggles select on current pid
- `k` / `K`: for each selected (or current), run kill with force flag; refresh list
- `q` quits
- Initial load: `list_processes` then spawn `std::thread` to fetch `listening_ports` and merge into app on next draw (channel `std::sync::mpsc`)

- [ ] **Step 4: Wire `Target::Tui => tui::run()?` in `main.rs` (`match cli.target`)**

- [ ] **Step 5: Manual test on macOS**

```bash
cargo run --
# verify list, search, select, quit
```

- [ ] **Step 6: Commit**

```bash
git add src/tui src/main.rs src/lib.rs
git commit -m "feat: add ratatui process browser with multi-select kill"
```

---

### Task 11: `sw ports` subcommand + README polish

**Files:**
- Create: `src/commands/ports_list.rs` (or extend `port.rs`)
- Modify: `src/main.rs`
- Modify: `README.md`

**Interfaces:**
- Produces: `fn run_ports_list() -> anyhow::Result<()>` printing LISTEN table (TUI optional later; MVP CLI table is enough if full ports TUI not ready — prefer simple CLI table for Priority A lite)

- [ ] **Step 1: Print listening table**

```rust
pub fn run_ports_list() -> anyhow::Result<()> {
    let mut procs = crate::process::list::list_processes();
    let ports = crate::process::ports::listening_ports()?;
    crate::process::ports::merge_ports(&mut procs, &ports);
    println!("PORT    PROCESS       PID");
    let mut rows = ports;
    rows.sort_by_key(|(port, _)| *port);
    for (port, pid) in rows {
        let name = procs
            .iter()
            .find(|p| p.pid == pid)
            .map(|p| p.name.as_str())
            .unwrap_or("?");
        println!("{port:<7} {name:<12} {pid}");
    }
    Ok(())
}
```

- [ ] **Step 2: Update README**

```markdown
# sweeper

Sweep unwanted processes away.

## Install (dev)

```bash
cargo install --path .
```

## Usage

```bash
sw           # TUI
sw node      # name
sw :3000     # port
sw top
sw clean
sw history
```
```

- [ ] **Step 3: Full test suite**

Run: `cargo test`
Expected: all PASS

- [ ] **Step 4: Commit**

```bash
git add src/commands src/main.rs README.md
git commit -m "feat: add ports list command and README usage"
```

---

## Self-Review (plan vs spec)

| Spec item | Task |
|---|---|
| Rust + clap + ratatui + sysinfo + nix + lsof | 1, 4–6, 10 |
| Target resolution | 2 |
| SIGTERM then optional SIGKILL | 6, 8 |
| Protected processes | 3 |
| History JSON capped | 7, 9 |
| TUI list/search/multi-kill | 10 |
| `sw node` / `sw :port` / `sw top` / `sw clean` | 8, 9 |
| No auto-kill clean | 9 |
| `sw ports` | 11 |
| Homebrew formula | Deferred (post-MVP packaging) |
| Project recognition / tree | Out of scope (spec §9) |

Homebrew is listed in the design as distribution intent; shipping a formula is deferred until the binary is stable — not required to close MVP functional scope.
