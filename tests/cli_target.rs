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
fn force_flag() {
    let cli = Cli::parse_from(["sw", ":3000", "--force"]);
    assert!(cli.force);
    assert_eq!(cli.target, Target::Ports(vec![3000]));
}
