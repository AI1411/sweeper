use std::collections::{HashMap, HashSet};

use crate::process::protect::is_protected;
use crate::process::tty::has_controlling_tty;
use crate::process::ProcessInfo;

/// A listener running longer than this is flagged as stale.
pub const STALE_SERVER_SECS: u64 = 4 * 60 * 60;
/// An idle listener must run at least this long before it is flagged.
pub const IDLE_LISTENER_SECS: u64 = 30 * 60;
pub const IDLE_CPU_THRESHOLD: f32 = 0.5;
/// Processes younger than this with recent CPU are treated as active dev sessions.
pub const ACTIVE_SESSION_MAX_SECS: u64 = 15 * 60;
pub const ACTIVE_CPU_THRESHOLD: f32 = 0.1;

const NAME_HINTS: &[&str] = &[
    "node",
    "bun",
    "deno",
    "vite",
    "next-server",
    "python",
    "java",
    "esbuild",
    "webpack",
    "playwright",
    "chrome",
    "chromium",
    "turbo",
    "jest",
    "vitest",
    "cargo",
    "rust-analyzer",
    "typescript-language-server",
    "pyright",
    "docker",
    "com.docker",
    "uvicorn",
    "pnpm",
    "astro",
    "eslint",
];

struct CmdPattern {
    stack: &'static str,
    patterns: &'static [&'static str],
}

const CMD_PATTERNS: &[CmdPattern] = &[
    CmdPattern {
        stack: "vite",
        patterns: &["node_modules/.bin/vite", "vite dev", "vite preview"],
    },
    CmdPattern {
        stack: "next",
        patterns: &["next dev", "next-server", "node_modules/.bin/next"],
    },
    CmdPattern {
        stack: "playwright",
        patterns: &["playwright", "ms-playwright", "--remote-debugging-port"],
    },
    CmdPattern {
        stack: "webpack",
        patterns: &["webpack-dev-server", "webpack serve"],
    },
    CmdPattern {
        stack: "esbuild",
        patterns: &["esbuild"],
    },
    CmdPattern {
        stack: "turbo",
        patterns: &["turbo run", "node_modules/.bin/turbo"],
    },
    CmdPattern {
        stack: "hono",
        patterns: &["hono"],
    },
    CmdPattern {
        stack: "jest",
        patterns: &["jest", "node_modules/.bin/jest"],
    },
    CmdPattern {
        stack: "vitest",
        patterns: &["vitest", "node_modules/.bin/vitest"],
    },
    CmdPattern {
        stack: "rust-analyzer",
        patterns: &["rust-analyzer"],
    },
    CmdPattern {
        stack: "typescript",
        patterns: &["typescript-language-server", "tsserver"],
    },
    CmdPattern {
        stack: "pyright",
        patterns: &["pyright", "pylsp", "python-language-server"],
    },
    CmdPattern {
        stack: "docker",
        patterns: &["docker-proxy", "com.docker"],
    },
    // e.g. `uvicorn main:app --reload`
    CmdPattern {
        stack: "uvicorn",
        patterns: &["uvicorn", "gunicorn"],
    },
    // e.g. `fastapi run` / uvicorn with fastapi app path
    CmdPattern {
        stack: "fastapi",
        patterns: &["fastapi", "uvicorn main:app"],
    },
    // e.g. `pnpm dev` / `node_modules/.bin/pnpm`
    CmdPattern {
        stack: "pnpm",
        patterns: &["pnpm dev", "pnpm run", "node_modules/.bin/pnpm"],
    },
    // e.g. `astro dev`
    CmdPattern {
        stack: "astro",
        patterns: &["astro dev", "node_modules/.bin/astro"],
    },
    // e.g. ESLint language server / `eslint --fix`
    CmdPattern {
        stack: "eslint",
        patterns: &["eslint", "vscode-eslint", "eslintServer"],
    },
];

#[derive(Debug, Clone, PartialEq)]
pub struct CleanCandidate {
    pub process: ProcessInfo,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CleanSummary {
    pub stale_servers: usize,
    pub orphans: usize,
    pub zombies: usize,
    pub idle_listeners: usize,
    pub listening: usize,
    pub estimated_bytes: u64,
}

pub fn propose_leftovers(procs: &[ProcessInfo], listening: &[(u16, u32)]) -> Vec<CleanCandidate> {
    let listen_pids: HashSet<u32> = listening.iter().map(|(_, pid)| *pid).collect();
    let pid_set: HashSet<u32> = procs.iter().map(|p| p.pid).collect();
    let ports_by_pid: HashMap<u32, Vec<u16>> =
        listening
            .iter()
            .fold(HashMap::new(), |mut acc, (port, pid)| {
                acc.entry(*pid).or_default().push(*port);
                acc
            });
    let by_pid: HashMap<u32, &ProcessInfo> = procs.iter().map(|p| (p.pid, p)).collect();

    let mut out: Vec<CleanCandidate> = procs
        .iter()
        .filter(|p| !is_protected(&p.name))
        .filter_map(|p| {
            let (stack, stack_reason) = detect_dev_stack(p)?;
            let parent = by_pid.get(&p.ppid).copied();
            let reasons = collect_reasons(
                p,
                parent,
                &by_pid,
                &listen_pids,
                &pid_set,
                &ports_by_pid,
                &stack_reason,
            );
            if reasons.is_empty() {
                return None;
            }
            if !is_candidate(&reasons) {
                return None;
            }
            let mut tagged = reasons;
            if !tagged
                .iter()
                .any(|r| r.starts_with("stack:") || r.starts_with("name:"))
            {
                tagged.insert(0, format!("stack:{stack}"));
            }
            Some(CleanCandidate {
                process: p.clone(),
                reasons: tagged,
            })
        })
        .collect();

    out.sort_by(compare_candidates);
    out
}

/// Sort clean candidates by confidence (high first), then score, then name/pid.
pub fn compare_candidates(a: &CleanCandidate, b: &CleanCandidate) -> std::cmp::Ordering {
    confidence_rank(confidence_level(b))
        .cmp(&confidence_rank(confidence_level(a)))
        .then_with(|| score_candidate(b).cmp(&score_candidate(a)))
        .then_with(|| a.process.name.cmp(&b.process.name))
        .then_with(|| a.process.pid.cmp(&b.process.pid))
}

fn confidence_rank(level: &str) -> u8 {
    match level {
        "high" => 3,
        "medium" => 2,
        _ => 1,
    }
}

pub fn summarize(candidates: &[CleanCandidate]) -> CleanSummary {
    let mut summary = CleanSummary::default();
    for c in candidates {
        summary.estimated_bytes += c.process.memory_bytes;
        if c.reasons.iter().any(|r| r == "stale-server") {
            summary.stale_servers += 1;
        }
        if c.reasons
            .iter()
            .any(|r| r == "orphan-ppid" || r == "orphan-parent" || r == "orphan-parent-defunct")
        {
            summary.orphans += 1;
        }
        if c.reasons.iter().any(|r| r == "zombie") {
            summary.zombies += 1;
        }
        if c.reasons.iter().any(|r| r == "idle-listener") {
            summary.idle_listeners += 1;
        }
        if c.reasons.iter().any(|r| r == "listening") {
            summary.listening += 1;
        }
    }
    summary
}

pub fn score_candidate(c: &CleanCandidate) -> u32 {
    let mut score = 0;
    for reason in &c.reasons {
        score += match reason.as_str() {
            "zombie" => 100,
            "orphan-parent" => 80,
            "orphan-parent-defunct" => 80,
            "orphan-ppid" => 70,
            "stale-server" => 60,
            "idle-listener" => 50,
            "listening" => 20,
            "dev-port" => 15,
            _ => 0,
        };
    }
    score
}

/// Confidence hint for CLI display derived from `score_candidate`.
pub fn confidence_level(c: &CleanCandidate) -> &'static str {
    let score = score_candidate(c);
    if score >= 60 {
        "high"
    } else if score >= 30 {
        "medium"
    } else {
        "low"
    }
}

fn detect_dev_stack(p: &ProcessInfo) -> Option<(String, String)> {
    if let Some(cmd) = p.command.as_deref() {
        let cmd_l = cmd.to_lowercase();
        for pat in CMD_PATTERNS {
            for needle in pat.patterns {
                if cmd_l.contains(needle) {
                    return Some((pat.stack.to_string(), format!("stack:{}", pat.stack)));
                }
            }
        }
    }

    if let Some(cwd) = p.cwd.as_deref() {
        let cwd_l = cwd.to_lowercase();
        if cwd_l.contains("node_modules") {
            return Some(("node".into(), "cwd:node_modules".into()));
        }
    }

    let name = p.name.to_lowercase();
    for hint in NAME_HINTS {
        if name.contains(hint) {
            return Some((hint.to_string(), format!("name:{hint}")));
        }
    }
    None
}

/// Heuristics for an in-progress dev session (not a leftover).
///
/// Documented rules (all require a young process unless noted):
///
/// 1. **Recent CPU** — `run_time < 15m` and `CPU >= 0.1%` → active.
///    Example: vite started 2m ago at 2% CPU → excluded from stale/orphan rules.
///
/// 2. **Interactive shell parent** — parent is `zsh`/`bash`/`fish`/… and process is young.
///    Example: `bash` → `node :3000` started 5m ago → excluded.
///
/// 3. **IDE-launched** — parent chain includes `Cursor`, `Code Helper`, `idea`, etc. and process is young.
///    Example: `Cursor Helper` → `node` vite 3m ago → excluded.
///
/// 4. **TTY-attached listener** — process is LISTEN, has a controlling TTY, and an interactive-shell ancestor.
///    Example: dev server in a terminal tab with `zsh` ancestor → excluded even past 15m if still TTY-bound.
///
/// Deferred: restart detection via history (same cwd + command hash within N minutes).
pub fn is_likely_active_session(
    p: &ProcessInfo,
    parent: Option<&ProcessInfo>,
    by_pid: &HashMap<u32, &ProcessInfo>,
    listening: bool,
) -> bool {
    let young = p.run_time_secs < ACTIVE_SESSION_MAX_SECS;
    if young && p.cpu >= ACTIVE_CPU_THRESHOLD {
        return true;
    }
    if young {
        if let Some(parent) = parent {
            if !parent.is_zombie && is_interactive_shell(&parent.name) {
                return true;
            }
        }
        if has_ide_ancestor(p, by_pid) {
            return true;
        }
    }
    if listening && has_controlling_tty(p.pid) && has_interactive_shell_ancestor(p, by_pid) {
        return true;
    }
    false
}

const IDE_LAUNCHERS: &[&str] = &[
    "cursor",
    "code helper",
    "code",
    "electron",
    "idea",
    "webstorm",
    "pycharm",
    "goland",
    "clion",
    "rider",
    "fleet",
    "zed",
    "windsurf",
];

fn is_ide_launcher(name: &str) -> bool {
    let n = name.to_lowercase();
    IDE_LAUNCHERS.iter().any(|hint| n.contains(hint))
}

fn has_ide_ancestor(p: &ProcessInfo, by_pid: &HashMap<u32, &ProcessInfo>) -> bool {
    let mut current_ppid = p.ppid;
    let mut seen = HashSet::new();
    while current_ppid != 0 && seen.insert(current_ppid) {
        let Some(parent) = by_pid.get(&current_ppid) else {
            break;
        };
        if is_ide_launcher(&parent.name) {
            return true;
        }
        current_ppid = parent.ppid;
    }
    false
}

fn has_interactive_shell_ancestor(p: &ProcessInfo, by_pid: &HashMap<u32, &ProcessInfo>) -> bool {
    let mut current_ppid = p.ppid;
    let mut seen = HashSet::new();
    while current_ppid != 0 && seen.insert(current_ppid) {
        let Some(parent) = by_pid.get(&current_ppid) else {
            break;
        };
        if !parent.is_zombie && is_interactive_shell(&parent.name) {
            return true;
        }
        current_ppid = parent.ppid;
    }
    false
}

fn is_interactive_shell(name: &str) -> bool {
    let n = name.to_lowercase();
    matches!(
        n.as_str(),
        "zsh" | "bash" | "fish" | "sh" | "dash" | "tmux" | "screen"
    )
}

/// Walk the parent chain; flag when an ancestor is missing from the snapshot or is zombie.
fn detect_orphan_chain(
    p: &ProcessInfo,
    by_pid: &HashMap<u32, &ProcessInfo>,
    pid_set: &HashSet<u32>,
) -> Option<&'static str> {
    let mut current_ppid = p.ppid;
    let mut seen = HashSet::new();
    while current_ppid != 0 && seen.insert(current_ppid) {
        if !pid_set.contains(&current_ppid) {
            return Some("orphan-parent");
        }
        let parent = by_pid.get(&current_ppid)?;
        if parent.is_zombie {
            return Some("orphan-parent-defunct");
        }
        current_ppid = parent.ppid;
    }
    None
}

fn collect_reasons(
    p: &ProcessInfo,
    parent: Option<&ProcessInfo>,
    by_pid: &HashMap<u32, &ProcessInfo>,
    listen_pids: &HashSet<u32>,
    pid_set: &HashSet<u32>,
    ports_by_pid: &HashMap<u32, Vec<u16>>,
    stack_reason: &str,
) -> Vec<String> {
    let mut reasons = vec![stack_reason.to_string()];
    let listening = listen_pids.contains(&p.pid);
    let active_session = is_likely_active_session(p, parent, by_pid, listening);
    let orphan_ppid = !active_session && (p.ppid == 0 || p.ppid == 1);
    let chain_orphan = if !active_session && !orphan_ppid {
        detect_orphan_chain(p, by_pid, pid_set)
    } else {
        None
    };
    let stale = !active_session && listening && p.run_time_secs >= STALE_SERVER_SECS;
    let idle_listener = !active_session
        && listening
        && p.run_time_secs >= IDLE_LISTENER_SECS
        && p.cpu < IDLE_CPU_THRESHOLD;
    let listen_ports = ports_by_pid.get(&p.pid).cloned().unwrap_or_default();
    let dev_port = p
        .ports
        .iter()
        .chain(listen_ports.iter())
        .any(|port| is_dev_port(*port));

    if p.is_zombie {
        reasons.push("zombie".into());
    }
    if orphan_ppid {
        reasons.push("orphan-ppid".into());
    }
    if let Some(tag) = chain_orphan {
        reasons.push(tag.into());
    }
    if stale {
        reasons.push("stale-server".into());
    }
    if idle_listener {
        reasons.push("idle-listener".into());
    }
    if listening {
        reasons.push("listening".into());
    }
    if dev_port {
        reasons.push("dev-port".into());
    }
    reasons
}

fn is_candidate(reasons: &[String]) -> bool {
    if reasons.iter().any(|r| r == "zombie") {
        return true;
    }
    if reasons.iter().any(|r| {
        matches!(
            r.as_str(),
            "orphan-ppid" | "orphan-parent" | "orphan-parent-defunct"
        )
    }) {
        return true;
    }
    if reasons
        .iter()
        .any(|r| r == "stale-server" || r == "idle-listener")
    {
        return true;
    }
    false
}

fn is_dev_port(port: u16) -> bool {
    matches!(
        port,
        3000 | 3001 | 3002 | 3003 | 4200 | 4321 | 5000 | 5173 | 8000 | 8080 | 8888 | 9000
    ) || (3000..=9999).contains(&port)
}

/// Drop candidates whose name, command, or pid string contains any exclude pattern (case-insensitive).
pub fn apply_excludes(cands: Vec<CleanCandidate>, excludes: &[String]) -> Vec<CleanCandidate> {
    if excludes.is_empty() {
        return cands;
    }
    let pats: Vec<String> = excludes.iter().map(|e| e.to_lowercase()).collect();
    cands
        .into_iter()
        .filter(|c| {
            let name = c.process.name.to_lowercase();
            let pid = c.process.pid.to_string();
            let cmd = c.process.command.as_deref().unwrap_or("").to_lowercase();
            !pats
                .iter()
                .any(|p| name.contains(p) || pid.contains(p.as_str()) || cmd.contains(p))
        })
        .collect()
}

pub fn excludes_from_env() -> Vec<String> {
    std::env::var("SWEEPER_CLEAN_EXCLUDE")
        .ok()
        .map(|v| {
            v.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// Human-readable reason strings for CLI display (machine tags stay in `reasons`).
pub fn format_reasons_display(c: &CleanCandidate) -> Vec<String> {
    c.reasons
        .iter()
        .map(|tag| format_reason_tag(tag, &c.process))
        .collect()
}

/// Truncate command line for display (default max 40 chars).
pub fn format_command_snippet(command: Option<&str>) -> Option<String> {
    const MAX: usize = 40;
    command.map(|cmd| {
        if cmd.len() <= MAX {
            cmd.to_string()
        } else {
            format!("{}…", cmd.chars().take(MAX).collect::<String>())
        }
    })
}

pub fn format_age(secs: u64) -> String {
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else {
        format!("{}h", secs / 3600)
    }
}

fn format_reason_tag(tag: &str, p: &ProcessInfo) -> String {
    match tag {
        "stale-server" => {
            let age = format_age(p.run_time_secs);
            match p.ports.first() {
                Some(port) => format!("stale-server ({age} on :{port})"),
                None => format!("stale-server ({age})"),
            }
        }
        "idle-listener" => format!(
            "idle-listener ({}m, CPU {:.1}%)",
            p.run_time_secs / 60,
            p.cpu
        ),
        "orphan-parent" => format!("orphan-parent (ppid {} missing)", p.ppid),
        "orphan-parent-defunct" => format!("orphan-parent (ppid {} defunct)", p.ppid),
        "orphan-ppid" => "orphan-ppid (launchd)".into(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proc_with(
        pid: u32,
        ppid: u32,
        name: &str,
        cpu: f32,
        run_time_secs: u64,
        ports: Vec<u16>,
        command: Option<&str>,
    ) -> ProcessInfo {
        ProcessInfo {
            pid,
            ppid,
            name: name.into(),
            cpu,
            memory_bytes: 0,
            ports,
            command: command.map(str::to_string),
            cwd: None,
            run_time_secs,
            is_zombie: false,
        }
    }

    fn zombie_proc(pid: u32, ppid: u32, name: &str, run_time_secs: u64) -> ProcessInfo {
        ProcessInfo {
            pid,
            ppid,
            name: name.into(),
            cpu: 0.0,
            memory_bytes: 0,
            ports: vec![],
            command: None,
            cwd: None,
            run_time_secs,
            is_zombie: true,
        }
    }

    #[test]
    fn detects_stack_from_command_line() {
        let p = proc_with(
            1,
            42,
            "node",
            0.0,
            60,
            vec![5173],
            Some("/usr/bin/node ./node_modules/.bin/vite --port 5173"),
        );
        let out = propose_leftovers(&[p], &[(5173, 1)]);
        assert_eq!(out.len(), 1);
        assert!(out[0].reasons.iter().any(|r| r == "stack:vite"));
        assert!(out[0].reasons.iter().any(|r| r == "dev-port"));
    }

    #[test]
    fn skips_active_listener_with_healthy_parent() {
        let procs = vec![
            proc_with(50, 1, "bash", 0.0, 3600, vec![], None),
            proc_with(100, 50, "node", 5.0, 120, vec![3000], None),
        ];
        let listening = vec![(3000, 100)];
        let out = propose_leftovers(&procs, &listening);
        assert!(out.is_empty());
    }

    #[test]
    fn proposes_stale_listener() {
        let procs = vec![proc_with(
            100,
            50,
            "node",
            0.0,
            STALE_SERVER_SECS,
            vec![3000],
            None,
        )];
        let listening = vec![(3000, 100)];
        let out = propose_leftovers(&procs, &listening);
        assert_eq!(out.len(), 1);
        assert!(out[0].reasons.iter().any(|r| r == "stale-server"));
    }

    #[test]
    fn proposes_idle_listener() {
        let procs = vec![proc_with(
            100,
            50,
            "node",
            0.0,
            IDLE_LISTENER_SECS,
            vec![3000],
            None,
        )];
        let listening = vec![(3000, 100)];
        let out = propose_leftovers(&procs, &listening);
        assert_eq!(out.len(), 1);
        assert!(out[0].reasons.iter().any(|r| r == "idle-listener"));
    }

    #[test]
    fn proposes_orphan_parent_missing_from_snapshot() {
        let procs = vec![proc_with(200, 9999, "node", 0.0, 60, vec![3000], None)];
        let listening = vec![(3000, 200)];
        let out = propose_leftovers(&procs, &listening);
        assert_eq!(out.len(), 1);
        assert!(out[0].reasons.iter().any(|r| r == "orphan-parent"));
    }

    #[test]
    fn formats_stale_server_reason_with_port() {
        let c = CleanCandidate {
            process: proc_with(1, 50, "node", 0.0, STALE_SERVER_SECS, vec![3000], None),
            reasons: vec!["stale-server".into()],
        };
        let out = format_reasons_display(&c);
        assert_eq!(
            out[0],
            format!("stale-server ({}h on :3000)", STALE_SERVER_SECS / 3600)
        );
    }

    #[test]
    fn formats_idle_listener_reason() {
        let c = CleanCandidate {
            process: proc_with(1, 50, "node", 0.1, IDLE_LISTENER_SECS, vec![3000], None),
            reasons: vec!["idle-listener".into()],
        };
        let out = format_reasons_display(&c);
        assert_eq!(
            out[0],
            format!("idle-listener ({}m, CPU 0.1%)", IDLE_LISTENER_SECS / 60)
        );
    }

    #[test]
    fn truncates_command_snippet() {
        let long = "node ./node_modules/.bin/vite --port 3000 --host";
        let s = format_command_snippet(Some(long)).unwrap();
        assert!(s.chars().count() <= 41);
        assert!(s.ends_with('…'));
    }

    #[test]
    fn confidence_high_for_stale_and_orphans() {
        let stale = CleanCandidate {
            process: proc_with(1, 50, "node", 0.0, STALE_SERVER_SECS, vec![3000], None),
            reasons: vec!["stale-server".into()],
        };
        assert_eq!(confidence_level(&stale), "high");
        let orphan = CleanCandidate {
            process: proc_with(1, 9999, "node", 0.0, 60, vec![3000], None),
            reasons: vec!["orphan-parent".into()],
        };
        assert_eq!(confidence_level(&orphan), "high");
    }

    #[test]
    fn confidence_medium_for_idle_listener() {
        let c = CleanCandidate {
            process: proc_with(1, 50, "node", 0.1, IDLE_LISTENER_SECS, vec![3000], None),
            reasons: vec!["idle-listener".into()],
        };
        assert_eq!(confidence_level(&c), "medium");
    }

    #[test]
    fn active_session_young_high_cpu() {
        let p = proc_with(1, 1, "node", 5.0, 120, vec![3000], None);
        let by_pid = HashMap::new();
        assert!(is_likely_active_session(&p, None, &by_pid, true));
    }

    #[test]
    fn active_session_young_shell_parent() {
        let parent = proc_with(42, 1, "zsh", 0.0, 3600, vec![], None);
        let child = proc_with(100, 42, "node", 0.0, 120, vec![3000], None);
        let by_pid: HashMap<u32, &ProcessInfo> = [(42, &parent)].into_iter().collect();
        assert!(is_likely_active_session(
            &child,
            Some(&parent),
            &by_pid,
            true
        ));
    }

    #[test]
    fn not_active_session_when_old() {
        let p = proc_with(1, 1, "node", 5.0, ACTIVE_SESSION_MAX_SECS, vec![3000], None);
        let by_pid = HashMap::new();
        assert!(!is_likely_active_session(&p, None, &by_pid, true));
    }

    #[test]
    fn active_session_ide_ancestor() {
        let ide = proc_with(10, 1, "Cursor Helper", 0.0, 3600, vec![], None);
        let child = proc_with(100, 10, "node", 0.0, 120, vec![3000], None);
        let by_pid: HashMap<u32, &ProcessInfo> = [(10, &ide)].into_iter().collect();
        assert!(is_likely_active_session(&child, Some(&ide), &by_pid, true));
    }

    #[test]
    fn candidates_sorted_by_confidence_then_score() {
        let stale = CleanCandidate {
            process: proc_with(1, 50, "node", 0.0, STALE_SERVER_SECS, vec![3000], None),
            reasons: vec!["stale-server".into()],
        };
        let idle = CleanCandidate {
            process: proc_with(2, 50, "bun", 0.1, IDLE_LISTENER_SECS, vec![8787], None),
            reasons: vec!["idle-listener".into()],
        };
        assert_eq!(confidence_level(&stale), "high");
        assert_eq!(confidence_level(&idle), "medium");
        let mut cands = [idle.clone(), stale.clone()];
        cands.sort_by(compare_candidates);
        assert_eq!(cands[0].process.pid, 1);
        assert_eq!(cands[1].process.pid, 2);
    }

    #[test]
    fn skips_young_launchd_listener_with_cpu() {
        let procs = vec![proc_with(400, 1, "node", 5.0, 120, vec![3000], None)];
        let listening = vec![(3000, 400)];
        let out = propose_leftovers(&procs, &listening);
        assert!(out.is_empty());
    }

    #[test]
    fn skips_young_listener_under_interactive_shell() {
        let procs = vec![
            proc_with(42, 1, "bash", 0.0, 3600, vec![], None),
            proc_with(400, 42, "node", 0.0, 120, vec![3000], None),
        ];
        let listening = vec![(3000, 400)];
        let out = propose_leftovers(&procs, &listening);
        assert!(out.is_empty());
    }

    #[test]
    fn proposes_orphan_when_parent_is_zombie_in_snapshot() {
        let procs = vec![
            zombie_proc(50, 1, "bash", 3600),
            proc_with(100, 50, "node", 0.0, 120, vec![3000], None),
        ];
        let listening = vec![(3000, 100)];
        let out = propose_leftovers(&procs, &listening);
        assert_eq!(out.len(), 1);
        assert!(out[0].reasons.iter().any(|r| r == "orphan-parent-defunct"));
        let display = format_reasons_display(&out[0]);
        assert!(display
            .iter()
            .any(|s| s == "orphan-parent (ppid 50 defunct)"));
    }

    #[test]
    fn proposes_nested_worker_when_grandparent_missing() {
        let procs = vec![proc_with(101, 100, "esbuild", 0.0, 60, vec![], None)];
        let listening = vec![];
        let out = propose_leftovers(&procs, &listening);
        assert_eq!(out.len(), 1);
        assert!(out[0].reasons.iter().any(|r| r == "orphan-parent"));
    }
}
