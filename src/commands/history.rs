use crate::history::{last_entry, load_entries};
use crate::style;

pub fn run_history(last: bool) -> anyhow::Result<()> {
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
