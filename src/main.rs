use sweeper::cli::{Cli, SubCommand, Target};
use sweeper::commands::{name, port, top};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let force = cli.force;
    match cli.target {
        Target::Name(q) => name::run_name(&q, force)?,
        Target::Ports(ps) => port::run_ports(&ps, force)?,
        Target::Sub(SubCommand::Top) => top::run_top()?,
        Target::Tui => {
            println!("TUI not implemented yet");
        }
        other => {
            println!("Not implemented yet: {other:?}");
        }
    }
    Ok(())
}
