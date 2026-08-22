use clap::{Parser, Subcommand as ClapSubcommand};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Tui,
    Name(String),
    Ports(Vec<u16>),
    Sub(SubCommand),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubCommand {
    Ports,
    Top,
    Clean { exclude: Vec<String> },
    History { last: bool },
    Project { name: Option<String> },
}

#[derive(Debug, Parser)]
#[command(name = "sw", about = "Sweep unwanted processes away")]
pub struct CliArgs {
    /// Use SIGKILL when needed / requested
    #[arg(long, global = true)]
    pub force: bool,

    /// Kill process tree (root + descendants via PPID)
    #[arg(long, global = true)]
    pub tree: bool,

    /// Show kill targets without sending signals
    #[arg(long = "dry-run", global = true)]
    pub dry_run: bool,

    #[command(subcommand)]
    subcommand: Option<RawSub>,

    /// Positional targets: names and/or :ports (flags after targets are allowed)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    raw_targets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cli {
    pub force: bool,
    pub tree: bool,
    pub dry_run: bool,
    pub target: Target,
}

#[derive(Debug, ClapSubcommand)]
enum RawSub {
    #[command(visible_alias = "p")]
    Ports,
    #[command(visible_alias = "t")]
    Top,
    #[command(visible_alias = "c")]
    Clean {
        /// Exclude candidates whose name or PID contains this pattern (repeatable)
        #[arg(long = "exclude", value_name = "PATTERN")]
        exclude: Vec<String>,
    },
    #[command(visible_alias = "h")]
    History {
        #[arg(long)]
        last: bool,
    },
    #[command(visible_alias = "proj")]
    Project { name: Option<String> },
}

impl Cli {
    pub fn parse_from<I, T>(itr: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        let args = CliArgs::parse_from(itr);
        Self::from_args(args)
    }

    pub fn parse() -> Self {
        Self::from_args(CliArgs::parse())
    }

    fn from_args(args: CliArgs) -> Self {
        let mut force = args.force;
        let mut tree = args.tree;
        let mut dry_run = args.dry_run;
        let mut raw_targets = Vec::new();
        for t in args.raw_targets {
            match t.as_str() {
                "--force" => force = true,
                "--tree" => tree = true,
                "--dry-run" => dry_run = true,
                _ => raw_targets.push(t),
            }
        }
        let target = resolve_target(args.subcommand, raw_targets);
        Self {
            force,
            tree,
            dry_run,
            target,
        }
    }
}

fn resolve_target(subcommand: Option<RawSub>, raw_targets: Vec<String>) -> Target {
    if let Some(sub) = subcommand {
        return match sub {
            RawSub::Ports => Target::Sub(SubCommand::Ports),
            RawSub::Top => Target::Sub(SubCommand::Top),
            RawSub::Clean { exclude } => Target::Sub(SubCommand::Clean { exclude }),
            RawSub::History { last } => Target::Sub(SubCommand::History { last }),
            RawSub::Project { name } => Target::Sub(SubCommand::Project { name }),
        };
    }

    if raw_targets.is_empty() {
        return Target::Tui;
    }

    let mut ports = Vec::new();
    let mut names = Vec::new();
    for t in raw_targets {
        if let Some(p) = t.strip_prefix(':') {
            if let Some(expanded) = parse_port_token(p) {
                ports.extend(expanded);
                continue;
            }
        }
        names.push(t);
    }

    if !ports.is_empty() && names.is_empty() {
        return Target::Ports(ports);
    }
    if ports.is_empty() && names.len() == 1 {
        return Target::Name(names.remove(0));
    }
    // Mixed or multiple names: treat first as name (MVP)
    Target::Name(names.into_iter().next().unwrap_or_default())
}

/// Parse `:3000` or inclusive range `3000-3010`.
fn parse_port_token(token: &str) -> Option<Vec<u16>> {
    if token.is_empty() {
        return None;
    }
    if let Some((start, end)) = token.split_once('-') {
        let start: u16 = start.parse().ok()?;
        let end: u16 = end.parse().ok()?;
        if start > end {
            return None;
        }
        return Some((start..=end).collect());
    }
    let port: u16 = token.parse().ok()?;
    Some(vec![port])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_port() {
        assert_eq!(parse_port_token("3000"), Some(vec![3000]));
    }

    #[test]
    fn parse_port_range_inclusive() {
        assert_eq!(parse_port_token("3000-3002"), Some(vec![3000, 3001, 3002]));
    }

    #[test]
    fn parse_invalid_range_returns_none() {
        assert_eq!(parse_port_token("3010-3000"), None);
        assert_eq!(parse_port_token(""), None);
    }
}
