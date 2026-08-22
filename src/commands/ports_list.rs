use crate::style;

pub fn run_ports_list() -> anyhow::Result<()> {
    let mut procs = crate::process::list::list_processes();
    let ports = crate::process::ports::listening_ports()?;
    crate::process::ports::merge_ports(&mut procs, &ports);
    println!(
        "{}    {}       {}",
        style::header("PORT"),
        style::header("PROCESS"),
        style::header("PID")
    );
    let mut rows = ports;
    rows.sort_by_key(|(port, _)| *port);
    for (port, pid) in rows {
        let name = procs
            .iter()
            .find(|p| p.pid == pid)
            .map(|p| p.name.as_str())
            .unwrap_or("?");
        println!(
            "{} {} {}",
            style::port(format!("{port:<7}")),
            style::process_name(format!("{name:<12}")),
            style::pid(pid)
        );
    }
    Ok(())
}
