use sweeper::cli::{Cli, SubCommand, Target};

#[test]
fn bare_sw_is_tui() {
    let cli = Cli::parse_from(["sw"]);
    assert_eq!(cli.target, Target::Tui);
    assert!(!cli.force);
}

#[test]
fn name_search() {
    let cli = Cli::parse_from(["sw", "node"]);
    assert_eq!(cli.target, Target::Name("node".into()));
}

#[test]
fn single_port() {
    let cli = Cli::parse_from(["sw", ":3000"]);
    assert_eq!(cli.target, Target::Ports(vec![3000]));
}

#[test]
fn multiple_ports() {
    let cli = Cli::parse_from(["sw", ":3000", ":3001"]);
    assert_eq!(cli.target, Target::Ports(vec![3000, 3001]));
}

#[test]
fn port_range() {
    let cli = Cli::parse_from(["sw", ":3000-3002"]);
    assert_eq!(cli.target, Target::Ports(vec![3000, 3001, 3002]));
}

#[test]
fn port_range_with_single() {
    let cli = Cli::parse_from(["sw", ":3000-3002", ":5173"]);
    assert_eq!(cli.target, Target::Ports(vec![3000, 3001, 3002, 5173]));
}

#[test]
fn subcommand_top() {
    let cli = Cli::parse_from(["sw", "top"]);
    assert_eq!(cli.target, Target::Sub(SubCommand::Top));
}

#[test]
fn subcommand_history_last() {
    let cli = Cli::parse_from(["sw", "history", "--last"]);
    assert_eq!(cli.target, Target::Sub(SubCommand::History { last: true }));
}

#[test]
fn short_alias_ports() {
    let cli = Cli::parse_from(["sw", "p"]);
    assert_eq!(cli.target, Target::Sub(SubCommand::Ports));
}

#[test]
fn short_alias_top() {
    let cli = Cli::parse_from(["sw", "t"]);
    assert_eq!(cli.target, Target::Sub(SubCommand::Top));
}

#[test]
fn short_alias_clean() {
    let cli = Cli::parse_from(["sw", "c"]);
    assert_eq!(
        cli.target,
        Target::Sub(SubCommand::Clean {
            exclude: Vec::new()
        })
    );
}

#[test]
fn clean_exclude_flag() {
    let cli = Cli::parse_from(["sw", "clean", "--exclude", "python", "--exclude", "1513"]);
    assert_eq!(
        cli.target,
        Target::Sub(SubCommand::Clean {
            exclude: vec!["python".into(), "1513".into()]
        })
    );
}

#[test]
fn short_alias_history_last() {
    let cli = Cli::parse_from(["sw", "h", "--last"]);
    assert_eq!(cli.target, Target::Sub(SubCommand::History { last: true }));
}

#[test]
fn short_alias_project() {
    let cli = Cli::parse_from(["sw", "proj", "my-app"]);
    assert_eq!(
        cli.target,
        Target::Sub(SubCommand::Project {
            name: Some("my-app".into())
        })
    );
}

#[test]
fn force_flag() {
    let cli = Cli::parse_from(["sw", ":3000", "--force"]);
    assert!(cli.force);
    assert_eq!(cli.target, Target::Ports(vec![3000]));
}

#[test]
fn force_flag_before_target() {
    let cli = Cli::parse_from(["sw", "--force", "node"]);
    assert!(cli.force);
    assert_eq!(cli.target, Target::Name("node".into()));
}

#[test]
fn tree_flag() {
    let cli = Cli::parse_from(["sw", "node", "--tree"]);
    assert!(cli.tree);
    assert_eq!(cli.target, Target::Name("node".into()));
}

#[test]
fn dry_run_flag() {
    let cli = Cli::parse_from(["sw", ":3000", "--dry-run"]);
    assert!(cli.dry_run);
    assert_eq!(cli.target, Target::Ports(vec![3000]));
}

#[test]
fn subcommand_ports() {
    let cli = Cli::parse_from(["sw", "ports"]);
    assert_eq!(cli.target, Target::Sub(SubCommand::Ports));
}

#[test]
fn subcommand_clean() {
    let cli = Cli::parse_from(["sw", "clean"]);
    assert_eq!(
        cli.target,
        Target::Sub(SubCommand::Clean {
            exclude: Vec::new()
        })
    );
}

#[test]
fn subcommand_history_default() {
    let cli = Cli::parse_from(["sw", "history"]);
    assert_eq!(cli.target, Target::Sub(SubCommand::History { last: false }));
}

#[test]
fn mixed_name_and_port_uses_first_name() {
    let cli = Cli::parse_from(["sw", "node", ":3000"]);
    assert_eq!(cli.target, Target::Name("node".into()));
}

#[test]
fn invalid_port_like_token_treated_as_name() {
    let cli = Cli::parse_from(["sw", ":notaport"]);
    assert_eq!(cli.target, Target::Name(":notaport".into()));
}

#[test]
fn subcommand_doctor() {
    let cli = Cli::parse_from(["sw", "doctor"]);
    assert_eq!(cli.target, Target::Sub(SubCommand::Doctor));
}

#[test]
fn subcommand_completions() {
    let cli = Cli::parse_from(["sw", "completions", "zsh"]);
    assert_eq!(
        cli.target,
        Target::Sub(SubCommand::Completions {
            shell: "zsh".into()
        })
    );
}
