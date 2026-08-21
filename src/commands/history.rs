use crate::history::{last_entry, load_entries};

pub fn run_history(last: bool) -> anyhow::Result<()> {
    if last {
        match last_entry()? {
            Some(e) => println!("{}  {}  PID {}  ports {:?}", e.time, e.name, e.pid, e.ports),
            None => println!("No history yet."),
        }
        return Ok(());
    }
    for e in load_entries()? {
        println!("{}  {}  PID {}  ports {:?}", e.time, e.name, e.pid, e.ports);
    }
    Ok(())
}
