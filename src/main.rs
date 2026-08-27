use sweeper::cli::{Cli, SubCommand, Target};
use sweeper::commands::{
    cache, clean, completions, disk, docker, doctor, history, memory, name, port, ports_list,
    project, top, watch,
};
use sweeper::json_output;
use sweeper::memory::MemorySort;
use sweeper::tui;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    if cli.json {
        json_output::prepare_json_mode();
    }
    let force = cli.force;
    let tree = cli.tree;
    let dry_run = cli.dry_run;
    let json = cli.json;
    match cli.target {
        Target::Name(q) => name::run_name(&q, force, tree, dry_run)?,
        Target::Ports(ps) => port::run_ports(&ps, force, tree, dry_run)?,
        Target::Sub(SubCommand::Ports) => ports_list::run_ports_list(json)?,
        Target::Sub(SubCommand::Top) => top::run_top(force, tree, dry_run, json)?,
        Target::Sub(SubCommand::Clean { exclude }) => {
            clean::run_clean(force, &exclude, dry_run, json)?
        }
        Target::Sub(SubCommand::History {
            last,
            project,
            since,
            limit,
        }) => history::run_history(history::HistoryOptions {
            last,
            project,
            since,
            limit,
            json,
        })?,
        Target::Sub(SubCommand::Project { name }) => {
            project::run_project(name, force, dry_run, json)?
        }
        Target::Sub(SubCommand::Disk { top }) => disk::run_disk(top, json)?,
        Target::Sub(SubCommand::Cache) => cache::run_cache(json)?,
        Target::Sub(SubCommand::Docker) => docker::run_docker(json)?,
        Target::Sub(SubCommand::Memory {
            action,
            sort,
            warn_above,
            leaks,
        }) => {
            let sort_field = match sort.as_deref() {
                None => MemorySort::default(),
                Some(s) => MemorySort::parse(s).ok_or_else(|| {
                    anyhow::anyhow!("invalid --sort value: {s} (use memory, name, or status)")
                })?,
            };
            memory::run_memory(action, sort_field, warn_above, leaks, dry_run, json)?
        }
        Target::Sub(SubCommand::Completions { shell }) => completions::run_completions(&shell)?,
        Target::Sub(SubCommand::Doctor) => doctor::run_doctor(json)?,
        Target::Sub(SubCommand::Watch {
            ports,
            until,
            interval,
            timeout,
            quiet,
        }) => {
            let until_mode = match until.as_str() {
                "free" => watch::WatchUntil::Free,
                _ => watch::WatchUntil::Listen,
            };
            watch::run_watch(watch::WatchOptions {
                ports,
                until: until_mode,
                interval_secs: interval,
                timeout_secs: timeout,
                quiet,
                json,
            })?
        }
        Target::Tui => tui::run()?,
    }
    Ok(())
}
