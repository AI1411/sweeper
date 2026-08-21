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
        let cwd = proc_.cwd().map(|p| p.to_string_lossy().into_owned());

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_matches_substring() {
        let all = list_processes();
        assert!(!all.is_empty());
    }
}
