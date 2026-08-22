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
    out
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
    fn list_processes_returns_something() {
        assert!(!list_processes().is_empty());
    }
}
