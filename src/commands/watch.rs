use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;

use crate::json_output::emit_json;
use crate::process::list::list_processes;
use crate::process::ports::{clear_port_cache, pids_for_port};
use crate::style;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchUntil {
    Listen,
    Free,
}

impl WatchUntil {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "listen" | "listening" => Some(Self::Listen),
            "free" => Some(Self::Free),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WatchOptions {
    pub ports: Vec<u16>,
    pub until: WatchUntil,
    pub interval_secs: f64,
    pub timeout_secs: Option<u64>,
    pub quiet: bool,
    pub json: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct WatchEventJson {
    pub event: String,
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process: Option<String>,
}

pub trait PortWatchResolver {
    fn listeners(&self, port: u16) -> anyhow::Result<Vec<u32>>;
}

pub struct LivePortWatchResolver;

impl PortWatchResolver for LivePortWatchResolver {
    fn listeners(&self, port: u16) -> anyhow::Result<Vec<u32>> {
        clear_port_cache();
        pids_for_port(port).map_err(|e| anyhow::anyhow!("{e}"))
    }
}

pub fn run_watch(opts: WatchOptions) -> anyhow::Result<()> {
    let resolver = LivePortWatchResolver;
    run_watch_with_resolver(&opts, &resolver)
}

pub fn run_watch_with_resolver(
    opts: &WatchOptions,
    resolver: &dyn PortWatchResolver,
) -> anyhow::Result<()> {
    if opts.ports.is_empty() {
        anyhow::bail!("watch requires at least one port (e.g. sw watch :3000)");
    }
    if opts.interval_secs < 0.5 {
        anyhow::bail!("--interval must be at least 0.5 seconds");
    }

    let mode_label = match opts.until {
        WatchUntil::Listen => "LISTEN",
        WatchUntil::Free => "free",
    };
    if !opts.quiet && !opts.json {
        let ports: Vec<String> = opts.ports.iter().map(|p| format!(":{p}")).collect();
        println!(
            "watching {} (wait for {}, interval {}s) …",
            ports.join(", "),
            mode_label,
            opts.interval_secs
        );
    }

    let started = Instant::now();
    let interval = Duration::from_secs_f64(opts.interval_secs);

    loop {
        if let Some(limit) = opts.timeout_secs {
            if started.elapsed() >= Duration::from_secs(limit) {
                let msg = format!("timeout after {limit}s waiting for ports to be {mode_label}");
                if opts.json {
                    emit_json(&serde_json::json!({ "error": msg, "timeout": true }))?;
                } else {
                    eprintln!("{}", style::error(msg));
                }
                std::process::exit(1);
            }
        }

        if let Some(snapshot) = evaluate_ports(&opts.ports, opts.until, resolver)? {
            return finish_success(opts, snapshot);
        }

        if !opts.quiet && !opts.json {
            let status = format_port_status(&opts.ports, opts.until, resolver)?;
            println!("… {status}");
        }

        thread::sleep(interval);
    }
}

#[derive(Debug, Clone)]
pub struct PortSnapshot {
    pub port: u16,
    pub pid: Option<u32>,
    pub process: Option<String>,
}

fn evaluate_ports(
    ports: &[u16],
    until: WatchUntil,
    resolver: &dyn PortWatchResolver,
) -> anyhow::Result<Option<Vec<PortSnapshot>>> {
    let mut snapshots = Vec::with_capacity(ports.len());
    for &port in ports {
        let pids = resolver.listeners(port)?;
        let listening = !pids.is_empty();
        let satisfied = match until {
            WatchUntil::Listen => listening,
            WatchUntil::Free => !listening,
        };
        if !satisfied {
            return Ok(None);
        }
        let pid = pids.first().copied();
        let process = pid.and_then(|p| {
            list_processes()
                .into_iter()
                .find(|proc| proc.pid == p)
                .map(|proc| proc.name)
        });
        snapshots.push(PortSnapshot { port, pid, process });
    }
    Ok(Some(snapshots))
}

fn format_port_status(
    ports: &[u16],
    until: WatchUntil,
    resolver: &dyn PortWatchResolver,
) -> anyhow::Result<String> {
    let mut parts = Vec::new();
    for &port in ports {
        let pids = resolver.listeners(port)?;
        let label = if pids.is_empty() {
            "free".into()
        } else {
            format!("listening pid {}", pids[0])
        };
        parts.push(format!(":{port} {label}"));
    }
    let _ = until;
    Ok(parts.join(", "))
}

fn finish_success(opts: &WatchOptions, snapshots: Vec<PortSnapshot>) -> anyhow::Result<()> {
    if opts.json {
        let events: Vec<WatchEventJson> = snapshots
            .iter()
            .map(|s| WatchEventJson {
                event: match opts.until {
                    WatchUntil::Listen => "listening".into(),
                    WatchUntil::Free => "free".into(),
                },
                port: s.port,
                pid: s.pid,
                process: s.process.clone(),
            })
            .collect();
        for event in &events {
            emit_json(event)?;
        }
        return Ok(());
    }

    for snap in &snapshots {
        match opts.until {
            WatchUntil::Listen => {
                let proc = snap.process.as_deref().unwrap_or("?");
                let pid = snap.pid.unwrap_or(0);
                let line = format!("✓ :{} listening — {} pid {}", snap.port, proc, pid);
                if opts.quiet {
                    println!("{line}");
                } else {
                    println!("{}", style::success(line));
                }
            }
            WatchUntil::Free => {
                let line = format!("✓ :{} free", snap.port);
                if opts.quiet {
                    println!("{line}");
                } else {
                    println!("{}", style::success(line));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    struct MockResolver {
        states: RefCell<HashMap<u16, Vec<Vec<u32>>>>,
        step: RefCell<usize>,
    }

    impl MockResolver {
        fn new(sequence: Vec<(u16, Vec<u32>)>) -> Self {
            let mut map: HashMap<u16, Vec<Vec<u32>>> = HashMap::new();
            for (port, pids) in sequence {
                map.entry(port).or_default().push(pids);
            }
            Self {
                states: RefCell::new(map),
                step: RefCell::new(0),
            }
        }
    }

    impl PortWatchResolver for MockResolver {
        fn listeners(&self, port: u16) -> anyhow::Result<Vec<u32>> {
            let step = *self.step.borrow();
            *self.step.borrow_mut() = step + 1;
            let states = self.states.borrow();
            let seq = states
                .get(&port)
                .ok_or_else(|| anyhow::anyhow!("no mock"))?;
            let idx = (step).min(seq.len().saturating_sub(1));
            Ok(seq[idx].clone())
        }
    }

    #[test]
    fn waits_until_listen() {
        let resolver = MockResolver::new(vec![(3000, vec![]), (3000, vec![42])]);
        let opts = WatchOptions {
            ports: vec![3000],
            until: WatchUntil::Listen,
            interval_secs: 0.5,
            timeout_secs: None,
            quiet: true,
            json: false,
        };
        run_watch_with_resolver(&opts, &resolver).unwrap();
    }

    #[test]
    fn waits_until_free() {
        let resolver = MockResolver::new(vec![(3000, vec![42]), (3000, vec![])]);
        let opts = WatchOptions {
            ports: vec![3000],
            until: WatchUntil::Free,
            interval_secs: 0.5,
            timeout_secs: None,
            quiet: true,
            json: false,
        };
        run_watch_with_resolver(&opts, &resolver).unwrap();
    }

    #[test]
    fn all_ports_must_listen() {
        struct TwoPortResolver {
            call: RefCell<usize>,
        }

        impl PortWatchResolver for TwoPortResolver {
            fn listeners(&self, port: u16) -> anyhow::Result<Vec<u32>> {
                let n = *self.call.borrow();
                *self.call.borrow_mut() = n + 1;
                match (n, port) {
                    (0, 3000) => Ok(vec![1]),
                    (1, 3001) => Ok(vec![]),
                    (2, 3000) => Ok(vec![1]),
                    (3, 3001) => Ok(vec![2]),
                    _ => Ok(vec![]),
                }
            }
        }

        let resolver = TwoPortResolver {
            call: RefCell::new(0),
        };
        let first = evaluate_ports(&[3000, 3001], WatchUntil::Listen, &resolver).unwrap();
        assert!(first.is_none());
        let second = evaluate_ports(&[3000, 3001], WatchUntil::Listen, &resolver).unwrap();
        assert!(second.is_some());
    }
}
