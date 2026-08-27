use crate::history::{filter_entries, load_entries, parse_since_duration, HistoryEntry};
use crate::json_output::{emit_json, HistoryJson};
use crate::style;

pub struct HistoryOptions {
    pub last: bool,
    pub project: Option<String>,
    pub since: Option<String>,
    pub limit: Option<usize>,
    pub json: bool,
}

pub fn run_history(opts: HistoryOptions) -> anyhow::Result<()> {
    let since_secs =
        match opts.since.as_deref() {
            None => None,
            Some(s) => Some(parse_since_duration(s).ok_or_else(|| {
                anyhow::anyhow!("invalid --since duration (use e.g. 1h, 30m, 2d)")
            })?),
        };

    let limit = if opts.last { Some(1) } else { opts.limit };
    let entries = filter_entries(load_entries()?, opts.project.as_deref(), since_secs, limit);

    if opts.json {
        let rows: Vec<HistoryJson> = entries
            .into_iter()
            .map(|e| HistoryJson {
                time: e.time,
                pid: e.pid,
                name: e.name,
                ports: e.ports,
                signal: format!("{:?}", e.signal).to_lowercase(),
                result: e.result,
                project: e.project,
            })
            .collect();
        return emit_json(&rows);
    }

    if entries.is_empty() {
        println!("{}", style::warn("No history yet."));
        return Ok(());
    }
    for e in entries {
        print_entry(&e);
    }
    Ok(())
}

fn print_entry(e: &HistoryEntry) {
    let project = e
        .project
        .as_deref()
        .map(|p| format!("  project:{p}"))
        .unwrap_or_default();
    println!(
        "{}  {}  {} {}  {} {:?}{}",
        style::dim(&e.time),
        style::process_name(&e.name),
        style::dim("PID"),
        style::pid(e.pid),
        style::dim("ports"),
        e.ports,
        style::dim(project)
    );
}
