use std::collections::{HashMap, HashSet};

use sysinfo::{ProcessStatus, ProcessesToUpdate, System};

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
        let cwd = proc_.cwd().map(|p| p.to_string_lossy().into_owned());

        let is_zombie = matches!(proc_.status(), ProcessStatus::Zombie);
        out.push(ProcessInfo {
            pid: pid.as_u32(),
            ppid: proc_.parent().map(|p| p.as_u32()).unwrap_or(0),
            name,
            cpu: proc_.cpu_usage(),
            memory_bytes: proc_.memory(),
            ports: Vec::new(),
            command: cmd,
            cwd,
            run_time_secs: proc_.run_time(),
            is_zombie,
        });
    }
    out.sort_by_key(|a| a.name.to_lowercase());
    sort_processes_for_display(&mut out);
    out
}

/// Developer-centric ordering: listeners first, then memory, CPU, name.
pub fn sort_processes_for_display(procs: &mut [ProcessInfo]) {
    sort_processes(procs, SortMode::Default);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Default,
    CpuDesc,
    MemoryDesc,
    NameAsc,
    PortFirst,
}

impl SortMode {
    pub fn cycle(self) -> Self {
        match self {
            SortMode::Default => SortMode::CpuDesc,
            SortMode::CpuDesc => SortMode::MemoryDesc,
            SortMode::MemoryDesc => SortMode::NameAsc,
            SortMode::NameAsc => SortMode::PortFirst,
            SortMode::PortFirst => SortMode::Default,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SortMode::Default => "default",
            SortMode::CpuDesc => "CPU",
            SortMode::MemoryDesc => "memory",
            SortMode::NameAsc => "name",
            SortMode::PortFirst => "port",
        }
    }
}

pub fn sort_processes(procs: &mut [ProcessInfo], mode: SortMode) {
    procs.sort_by(|a, b| compare_processes(a, b, mode));
}

pub fn compare_processes(a: &ProcessInfo, b: &ProcessInfo, mode: SortMode) -> std::cmp::Ordering {
    match mode {
        SortMode::Default => {
            let a_listen = !a.ports.is_empty();
            let b_listen = !b.ports.is_empty();
            b_listen
                .cmp(&a_listen)
                .then_with(|| b.memory_bytes.cmp(&a.memory_bytes))
                .then_with(|| b.cpu.total_cmp(&a.cpu))
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                .then_with(|| a.pid.cmp(&b.pid))
        }
        SortMode::CpuDesc => b
            .cpu
            .total_cmp(&a.cpu)
            .then_with(|| b.memory_bytes.cmp(&a.memory_bytes))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            .then_with(|| a.pid.cmp(&b.pid)),
        SortMode::MemoryDesc => b
            .memory_bytes
            .cmp(&a.memory_bytes)
            .then_with(|| b.cpu.total_cmp(&a.cpu))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            .then_with(|| a.pid.cmp(&b.pid)),
        SortMode::NameAsc => a
            .name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then_with(|| a.pid.cmp(&b.pid)),
        SortMode::PortFirst => {
            let a_port = a.ports.first().copied().unwrap_or(u16::MAX);
            let b_port = b.ports.first().copied().unwrap_or(u16::MAX);
            a_port
                .cmp(&b_port)
                .then_with(|| b.memory_bytes.cmp(&a.memory_bytes))
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                .then_with(|| a.pid.cmp(&b.pid))
        }
    }
}

/// Refresh CPU, memory, and liveness for an existing snapshot; add new PIDs and drop exited ones.
pub fn refresh_process_list(procs: &mut Vec<ProcessInfo>) {
    let mut sys = System::new_all();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let mut fresh_map: HashMap<u32, ProcessInfo> = HashMap::new();
    for (pid, proc_) in sys.processes() {
        let pid_u32 = pid.as_u32();
        fresh_map.insert(
            pid_u32,
            ProcessInfo {
                pid: pid_u32,
                ppid: proc_.parent().map(|p| p.as_u32()).unwrap_or(0),
                name: proc_.name().to_string_lossy().into_owned(),
                cpu: proc_.cpu_usage(),
                memory_bytes: proc_.memory(),
                ports: Vec::new(),
                command: {
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
                },
                cwd: proc_.cwd().map(|p| p.to_string_lossy().into_owned()),
                run_time_secs: proc_.run_time(),
                is_zombie: matches!(proc_.status(), ProcessStatus::Zombie),
            },
        );
    }

    for p in procs.iter_mut() {
        if let Some(fresh) = fresh_map.get(&p.pid) {
            p.ppid = fresh.ppid;
            p.name.clone_from(&fresh.name);
            p.cpu = fresh.cpu;
            p.memory_bytes = fresh.memory_bytes;
            p.command.clone_from(&fresh.command);
            p.cwd.clone_from(&fresh.cwd);
            p.run_time_secs = fresh.run_time_secs;
            p.is_zombie = fresh.is_zombie;
        }
    }

    let alive: HashSet<u32> = fresh_map.keys().copied().collect();
    let existing: HashSet<u32> = procs.iter().map(|p| p.pid).collect();
    for (pid, fresh) in fresh_map {
        if !existing.contains(&pid) {
            procs.push(fresh);
        }
    }
    procs.retain(|p| alive.contains(&p.pid));
}

/// Case-insensitive substring match against process name or command line.
pub fn name_matches(query: &str, name: &str, command: Option<&str>) -> bool {
    let q = query.to_lowercase();
    name.to_lowercase().contains(&q)
        || command
            .map(|c| c.to_lowercase().contains(&q))
            .unwrap_or(false)
}

pub fn find_by_name_fuzzy(query: &str) -> Vec<ProcessInfo> {
    list_processes()
        .into_iter()
        .filter(|p| name_matches(query, &p.name, p.command.as_deref()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_matches_process_name() {
        assert!(name_matches("node", "node", None));
        assert!(name_matches("NOD", "node", None));
        assert!(!name_matches("python", "node", None));
    }

    #[test]
    fn name_matches_command_line() {
        assert!(name_matches(
            "vite",
            "node",
            Some("/usr/bin/node ./node_modules/.bin/vite")
        ));
        assert!(!name_matches("vite", "node", Some("node server.js")));
    }

    #[test]
    fn sort_cpu_desc_orders_by_cpu() {
        let mut procs = vec![
            ProcessInfo {
                pid: 1,
                ppid: 0,
                name: "a".into(),
                cpu: 1.0,
                memory_bytes: 500,
                ports: vec![],
                command: None,
                cwd: None,
                run_time_secs: 0,
                is_zombie: false,
            },
            ProcessInfo {
                pid: 2,
                ppid: 1,
                name: "b".into(),
                cpu: 5.0,
                memory_bytes: 1000,
                ports: vec![],
                command: None,
                cwd: None,
                run_time_secs: 0,
                is_zombie: false,
            },
        ];
        sort_processes(&mut procs, SortMode::CpuDesc);
        assert_eq!(procs[0].pid, 2);
    }

    #[test]
    fn sort_memory_desc_orders_by_memory() {
        let mut procs = vec![
            ProcessInfo {
                pid: 1,
                ppid: 0,
                name: "a".into(),
                cpu: 1.0,
                memory_bytes: 500,
                ports: vec![],
                command: None,
                cwd: None,
                run_time_secs: 0,
                is_zombie: false,
            },
            ProcessInfo {
                pid: 2,
                ppid: 1,
                name: "b".into(),
                cpu: 0.1,
                memory_bytes: 9000,
                ports: vec![],
                command: None,
                cwd: None,
                run_time_secs: 0,
                is_zombie: false,
            },
        ];
        sort_processes(&mut procs, SortMode::MemoryDesc);
        assert_eq!(procs[0].pid, 2);
    }

    #[test]
    fn sort_name_asc_orders_alphabetically() {
        let mut procs = vec![
            ProcessInfo {
                pid: 1,
                ppid: 0,
                name: "zebra".into(),
                cpu: 0.0,
                memory_bytes: 0,
                ports: vec![],
                command: None,
                cwd: None,
                run_time_secs: 0,
                is_zombie: false,
            },
            ProcessInfo {
                pid: 2,
                ppid: 1,
                name: "alpha".into(),
                cpu: 0.0,
                memory_bytes: 0,
                ports: vec![],
                command: None,
                cwd: None,
                run_time_secs: 0,
                is_zombie: false,
            },
        ];
        sort_processes(&mut procs, SortMode::NameAsc);
        assert_eq!(procs[0].name, "alpha");
    }

    #[test]
    fn sort_port_first_orders_by_lowest_port() {
        let mut procs = vec![
            ProcessInfo {
                pid: 1,
                ppid: 0,
                name: "a".into(),
                cpu: 0.0,
                memory_bytes: 0,
                ports: vec![8080],
                command: None,
                cwd: None,
                run_time_secs: 0,
                is_zombie: false,
            },
            ProcessInfo {
                pid: 2,
                ppid: 1,
                name: "b".into(),
                cpu: 0.0,
                memory_bytes: 0,
                ports: vec![3000],
                command: None,
                cwd: None,
                run_time_secs: 0,
                is_zombie: false,
            },
        ];
        sort_processes(&mut procs, SortMode::PortFirst);
        assert_eq!(procs[0].pid, 2);
    }

    #[test]
    fn sort_mode_cycles() {
        assert_eq!(SortMode::Default.cycle(), SortMode::CpuDesc);
        assert_eq!(SortMode::PortFirst.cycle(), SortMode::Default);
    }

    #[test]
    fn sort_processes_for_display_orders_listeners_first() {
        let mut procs = vec![
            ProcessInfo {
                pid: 1,
                ppid: 0,
                name: "bash".into(),
                cpu: 1.0,
                memory_bytes: 500,
                ports: vec![],
                command: None,
                cwd: None,
                run_time_secs: 0,
                is_zombie: false,
            },
            ProcessInfo {
                pid: 2,
                ppid: 1,
                name: "node".into(),
                cpu: 0.1,
                memory_bytes: 1000,
                ports: vec![3000],
                command: None,
                cwd: None,
                run_time_secs: 0,
                is_zombie: false,
            },
        ];
        sort_processes_for_display(&mut procs);
        assert_eq!(procs[0].pid, 2);
    }

    #[test]
    fn refresh_process_list_keeps_known_pid() {
        let mut procs = list_processes();
        if procs.is_empty() {
            return;
        }
        let pid = procs[0].pid;
        refresh_process_list(&mut procs);
        assert!(procs.iter().any(|p| p.pid == pid));
    }

    #[test]
    fn list_processes_returns_something() {
        assert!(!list_processes().is_empty());
    }
}
