use sweeper::cli::{Cli, SubCommand, Target};
use sweeper::commands::{clean, history, name, port, ports_list, project, top};
use sweeper::tui;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let force = cli.force;
    match cli.target {
        Target::Name(q) => name::run_name(&q, force)?,
        Target::Ports(ps) => port::run_ports(&ps, force)?,
        Target::Sub(SubCommand::Ports) => ports_list::run_ports_list()?,
        Target::Sub(SubCommand::Top) => top::run_top()?,
        Target::Sub(SubCommand::Clean) => clean::run_clean(force)?,
        Target::Sub(SubCommand::History { last }) => history::run_history(last)?,
        Target::Sub(SubCommand::Project { name }) => project::run_project(name, force)?,
        Target::Tui => tui::run()?,
    }
    Ok(())
}
