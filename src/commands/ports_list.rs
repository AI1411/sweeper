use crate::json_output::{emit_json, PortRow};
use crate::style;

pub fn format_ports_table(rows: &[(u16, u32)], procs: &[crate::process::ProcessInfo]) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    writeln!(
        out,
        "{}    {}       {}",
        style::header("PORT"),
        style::header("PROCESS"),
        style::header("PID")
    )
    .unwrap();
    let mut sorted = rows.to_vec();
    sorted.sort_by_key(|(port, _)| *port);
    for (port, pid) in sorted {
        let name = procs
            .iter()
            .find(|p| p.pid == pid)
            .map(|p| p.name.as_str())
            .unwrap_or("?");
        writeln!(
            out,
            "{} {} {}",
            style::port(format!("{port:<7}")),
            style::process_name(format!("{name:<12}")),
            style::pid(pid)
        )
        .unwrap();
    }
    out
}

pub fn run_ports_list(json: bool) -> anyhow::Result<()> {
    let mut procs = crate::process::list::list_processes();
    let ports = crate::process::ports::listening_ports()?;
    crate::process::ports::merge_ports(&mut procs, &ports);
    if json {
        let mut rows: Vec<PortRow> = Vec::new();
        for (port, pid) in ports {
            let process = procs
                .iter()
                .find(|p| p.pid == pid)
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "?".into());
            rows.push(PortRow { port, pid, process });
        }
        rows.sort_by_key(|r| r.port);
        return emit_json(&rows);
    }
    print!("{}", format_ports_table(&ports, &procs));
    Ok(())
}
