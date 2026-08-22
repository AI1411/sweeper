use sweeper::cli::{Cli, SubCommand, Target};
use sweeper::commands::{clean, history, name, port, ports_list, project, top};
use sweeper::tui;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let force = cli.force;
    let tree = cli.tree;
    let dry_run = cli.dry_run;
    match cli.target {
        Target::Name(q) => name::run_name(&q, force, tree, dry_run)?,
        Target::Ports(ps) => port::run_ports(&ps, force, tree, dry_run)?,
        Target::Sub(SubCommand::Ports) => ports_list::run_ports_list()?,
        Target::Sub(SubCommand::Top) => top::run_top()?,
        Target::Sub(SubCommand::Clean { exclude }) => clean::run_clean(force, &exclude, dry_run)?,
        Target::Sub(SubCommand::History { last }) => history::run_history(last)?,
        Target::Sub(SubCommand::Project { name }) => project::run_project(name, force, dry_run)?,
        Target::Tui => tui::run()?,
    }
    Ok(())
}
