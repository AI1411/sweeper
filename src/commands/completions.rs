use clap::CommandFactory;
use clap_complete::{generate, Shell};

use crate::cli::CliArgs;

pub fn run_completions(shell: &str) -> anyhow::Result<()> {
    let mut cmd = CliArgs::command();
    let shell = match shell {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        other => {
            anyhow::bail!("unsupported shell '{other}' (use bash, zsh, or fish)");
        }
    };
    generate(shell, &mut cmd, "sw", &mut std::io::stdout());
    Ok(())
}
