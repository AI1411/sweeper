use crate::history::{last_entry, load_entries};
use crate::json_output::{emit_json, HistoryJson};
use crate::style;

pub fn run_history(last: bool, json: bool) -> anyhow::Result<()> {
    if json {
        let entries = if last {
            last_entry()?.map(|e| vec![e]).unwrap_or_default()
        } else {
            load_entries()?
        };
        let rows: Vec<HistoryJson> = entries
            .into_iter()
            .map(|e| HistoryJson {
                time: e.time,
                pid: e.pid,
                name: e.name,
                ports: e.ports,
                signal: format!("{:?}", e.signal).to_lowercase(),
                result: e.result,
            })
            .collect();
        return emit_json(&rows);
    }
    if last {
        match last_entry()? {
            Some(e) => println!(
                "{}  {}  {} {}  {} {:?}",
                style::dim(&e.time),
                style::process_name(&e.name),
                style::dim("PID"),
                style::pid(e.pid),
                style::dim("ports"),
                e.ports
            ),
            None => println!("{}", style::warn("No history yet.")),
        }
        return Ok(());
    }
    let entries = load_entries()?;
    if entries.is_empty() {
        println!("{}", style::warn("No history yet."));
        return Ok(());
    }
    for e in entries {
        println!(
            "{}  {}  {} {}  {} {:?}",
            style::dim(&e.time),
            style::process_name(&e.name),
            style::dim("PID"),
            style::pid(e.pid),
            style::dim("ports"),
            e.ports
        );
    }
    Ok(())
}
