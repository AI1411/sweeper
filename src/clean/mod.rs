use std::collections::{HashMap, HashSet};

use crate::process::protect::is_protected;
use crate::process::ProcessInfo;

/// A listener running longer than this is flagged as stale.
pub const STALE_SERVER_SECS: u64 = 4 * 60 * 60;
/// An idle listener must run at least this long before it is flagged.
pub const IDLE_LISTENER_SECS: u64 = 30 * 60;
pub const IDLE_CPU_THRESHOLD: f32 = 0.5;

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

    let mut out: Vec<CleanCandidate> = procs
        .iter()
        .filter(|p| !is_protected(&p.name))
        .filter_map(|p| {
            let (stack, stack_reason) = detect_dev_stack(p)?;
            let reasons = collect_reasons(p, &listen_pids, &pid_set, &ports_by_pid, &stack_reason);
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

    out.sort_by(|a, b| {
        score_candidate(b)
            .cmp(&score_candidate(a))
            .then_with(|| a.process.name.cmp(&b.process.name))
            .then_with(|| a.process.pid.cmp(&b.process.pid))
    });
    out
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
            .any(|r| r == "orphan-ppid" || r == "orphan-parent")
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

fn collect_reasons(
    p: &ProcessInfo,
    listen_pids: &HashSet<u32>,
    pid_set: &HashSet<u32>,
    ports_by_pid: &HashMap<u32, Vec<u16>>,
    stack_reason: &str,
) -> Vec<String> {
    let mut reasons = vec![stack_reason.to_string()];
    let listening = listen_pids.contains(&p.pid);
    let orphan_ppid = p.ppid == 0 || p.ppid == 1;
    let orphan_parent = p.ppid != 0 && !pid_set.contains(&p.ppid);
    let stale = listening && p.run_time_secs >= STALE_SERVER_SECS;
    let idle_listener =
        listening && p.run_time_secs >= IDLE_LISTENER_SECS && p.cpu < IDLE_CPU_THRESHOLD;
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
    if orphan_parent {
        reasons.push("orphan-parent".into());
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
    if reasons
        .iter()
        .any(|r| r == "orphan-ppid" || r == "orphan-parent")
    {
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
}
