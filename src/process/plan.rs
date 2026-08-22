use crate::process::protect::is_protected;
use crate::process::tree::collect_tree_pids;
use crate::process::ProcessInfo;
use crate::style;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedKill {
    pub pid: u32,
    pub name: String,
    pub ports: Vec<u16>,
    pub memory_bytes: u64,
    pub protected: bool,
}

pub fn plan_kills(procs: &[ProcessInfo], roots: &[u32], tree: bool) -> Vec<PlannedKill> {
    let pids = if tree {
        collect_tree_pids(procs, roots)
    } else {
        roots.to_vec()
    };
    pids.into_iter()
        .map(|pid| {
            let info = procs.iter().find(|p| p.pid == pid);
            let name = info.map(|p| p.name.clone()).unwrap_or_else(|| "?".into());
            let protected = is_protected(&name);
            PlannedKill {
                pid,
                name,
                ports: info.map(|p| p.ports.clone()).unwrap_or_default(),
                memory_bytes: info.map(|p| p.memory_bytes).unwrap_or(0),
                protected,
            }
        })
        .collect()
}

pub fn print_dry_run(planned: &[PlannedKill], tree: bool) {
    let actionable = planned.iter().filter(|p| !p.protected).count();
    let tree_hint = if tree { " (+ descendants)" } else { "" };
    println!(
        "{} would terminate {} process(es){}",
        style::header("Dry run"),
        style::process_name(actionable),
        style::dim(tree_hint)
    );
    for p in planned {
        if p.protected {
            println!(
                "  {} {} {} {} {}",
                style::dim("skip"),
                style::process_name(&p.name),
                style::dim("pid"),
                style::pid(p.pid),
                style::dim("(protected)")
            );
            continue;
        }
        let ports = if p.ports.is_empty() {
            String::new()
        } else {
            let list = p
                .ports
                .iter()
                .map(|port| format!(":{port}"))
                .collect::<Vec<_>>()
                .join(",");
            format!(" ports {list}")
        };
        println!(
            "  {} {} {} {}",
            style::process_name(&p.name),
            style::dim("pid"),
            style::pid(p.pid),
            ports
        );
    }
    println!("{}", style::dim("No signals sent."));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::ProcessInfo;

    fn proc(pid: u32, name: &str) -> ProcessInfo {
        ProcessInfo {
            pid,
            ppid: 1,
            name: name.into(),
            cpu: 0.0,
            memory_bytes: 100,
            ports: vec![3000],
            command: None,
            cwd: None,
            run_time_secs: 0,
            is_zombie: false,
        }
    }

    #[test]
    fn plan_marks_protected_processes() {
        let procs = vec![proc(1, "node"), proc(2, "launchd")];
        let planned = plan_kills(&procs, &[1, 2], false);
        assert_eq!(planned.len(), 2);
        assert!(!planned[0].protected);
        assert!(planned[1].protected);
    }

    #[test]
    fn plan_tree_includes_child() {
        let procs = vec![
            ProcessInfo {
                pid: 10,
                ppid: 1,
                name: "node".into(),
                cpu: 0.0,
                memory_bytes: 0,
                ports: vec![],
                command: None,
                cwd: None,
                run_time_secs: 0,
                is_zombie: false,
            },
            ProcessInfo {
                pid: 11,
                ppid: 10,
                name: "worker".into(),
                cpu: 0.0,
                memory_bytes: 0,
                ports: vec![],
                command: None,
                cwd: None,
                run_time_secs: 0,
                is_zombie: false,
            },
        ];
        let planned = plan_kills(&procs, &[10], true);
        assert_eq!(planned.len(), 2);
        assert!(planned.iter().any(|p| p.pid == 11));
    }
}
