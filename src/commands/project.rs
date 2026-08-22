use crate::history::{append_entry, HistoryEntry, KillSignal};
use crate::process::kill::{kill_pid, KillOutcome};
use crate::process::list::list_processes;
use crate::process::ports::{listening_ports, merge_ports};
use crate::project::{find_projects_by_name, group_projects, ProjectGroup};
use crate::style;

use super::confirm::confirm;

pub fn run_project(name: Option<String>, force: bool) -> anyhow::Result<()> {
    let mut procs = list_processes();
    let ports = listening_ports().unwrap_or_default();
    merge_ports(&mut procs, &ports);
    let groups = group_projects(&procs);

    match name {
        None => list_all(&groups),
        Some(q) => kill_named(&groups, &q, force),
    }
}

fn list_all(groups: &[ProjectGroup]) -> anyhow::Result<()> {
    if groups.is_empty() {
        println!(
            "{}",
            style::warn("No projects inferred from process cwd/command.")
        );
        return Ok(());
    }
    println!("{}\n", style::header("Projects"));
    for g in groups {
        let mem: u64 = g.processes.iter().map(|p| p.memory_bytes).sum();
        let mut ports: Vec<u16> = g.processes.iter().flat_map(|p| p.ports.clone()).collect();
        ports.sort_unstable();
        ports.dedup();
        let port_str = if ports.is_empty() {
            "-".into()
        } else {
            ports
                .iter()
                .map(|p| format!(":{p}"))
                .collect::<Vec<_>>()
                .join(",")
        };
        println!(
            "{}  {}  {}  {}  {}",
            style::process_name(format!("{:<20}", g.name)),
            style::dim(&g.path),
            style::dim(format!("{} procs", g.processes.len())),
            style::mem(format!("{:.0} MB", mem as f64 / (1024.0 * 1024.0))),
            style::port(port_str)
        );
    }
    println!(
        "\n{}",
        style::dim("Tip: sw project <name>  to inspect and kill a project")
    );
    Ok(())
}

fn kill_named(groups: &[ProjectGroup], query: &str, force: bool) -> anyhow::Result<()> {
    let hits = find_projects_by_name(groups, query);
    if hits.is_empty() {
        println!("{}", style::warn(format!("No project matching '{query}'")));
        return Ok(());
    }
    if hits.len() > 1 {
        println!(
            "{}",
            style::warn(format!(
                "Multiple projects match '{query}'; be more specific:"
            ))
        );
        for g in &hits {
            println!(
                "  {}  {}",
                style::process_name(&g.name),
                style::dim(&g.path)
            );
        }
        return Ok(());
    }
    let g = hits[0];
    println!(
        "{} {}  {}\n",
        style::header("Project"),
        style::process_name(&g.name),
        style::dim(&g.path)
    );
    for p in &g.processes {
        let ports = if p.ports.is_empty() {
            "-".into()
        } else {
            p.ports
                .iter()
                .map(|port| format!(":{port}"))
                .collect::<Vec<_>>()
                .join(",")
        };
        println!(
            "  {}  {}  {}  {}",
            style::pid(format!("{:>6}", p.pid)),
            style::process_name(format!("{:<16}", p.name)),
            style::port(format!("{:<12}", ports)),
            style::mem(format!("{:.0} MB", p.memory_mb()))
        );
    }
    let total: u64 = g.processes.iter().map(|p| p.memory_bytes).sum();
    println!(
        "\n{} {}  {} {}",
        style::dim("Total:"),
        style::process_name(format!("{} processes", g.processes.len())),
        style::dim("memory"),
        style::mem(format!("{:.1} GB", total as f64 / 1e9))
    );
    if !confirm(&format!("Kill all processes in project '{}'?", g.name))? {
        println!("{}", style::warn("Cancelled."));
        return Ok(());
    }
    let mut outcomes = Vec::new();
    for p in &g.processes {
        let mut use_force = force;
        let mut outcome = kill_pid(p.pid, &p.name, use_force)?;
        if outcome == KillOutcome::StillAlive
            && !use_force
            && confirm(&format!("PID {} still alive. Force kill?", p.pid))?
        {
            use_force = true;
            outcome = kill_pid(p.pid, &p.name, true)?;
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
        println!(
            "{} {} {}: {}",
            style::process_name(&p.name),
            style::dim("pid"),
            style::pid(p.pid),
            style::kill_outcome(outcome)
        );
        outcomes.push(crate::report::KillResult::new(p.memory_bytes, p.ports.clone(), outcome));
    }
    crate::report::print_kill_summary(&outcomes);
    Ok(())
}
