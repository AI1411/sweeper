use std::path::Path;
use std::process::Command;

use serde::Serialize;

use crate::history::history_path;
use crate::json_output::emit_json;
use crate::process::ports_native;
use crate::process::protect::protect_config_path;
use crate::style;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticCheck {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
}

#[derive(Debug, Serialize)]
struct DoctorJson {
    checks: Vec<DiagnosticCheck>,
}

pub fn run_doctor(json: bool) -> anyhow::Result<()> {
    let checks = collect_checks();
    let failures = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Fail)
        .count();
    let warnings = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Warn)
        .count();

    if json {
        emit_json(&DoctorJson { checks })?;
    } else {
        println!("{}\n", style::header("Sweeper doctor"));
        for check in &checks {
            let icon = match check.status {
                CheckStatus::Pass => style::success("✓"),
                CheckStatus::Warn => style::warn("⚠"),
                CheckStatus::Fail => style::error("✗"),
            };
            println!("  {}  {}  {}", icon, check.name, check.message);
        }
        println!();
        if failures == 0 && warnings == 0 {
            println!("{}", style::success("All checks passed"));
        } else {
            println!(
                "{} warning{}, {} failure{}",
                warnings,
                if warnings == 1 { "" } else { "s" },
                failures,
                if failures == 1 { "" } else { "s" }
            );
        }
    }

    if failures > 0 {
        std::process::exit(1);
    }
    Ok(())
}

pub fn collect_checks() -> Vec<DiagnosticCheck> {
    vec![
        check_binary(),
        check_native_ports(),
        check_lsof(),
        check_history_path(),
        check_protect_config(),
        check_orb_cli(),
        check_docker_cli(),
        check_tty_color(),
    ]
}

fn check_binary() -> DiagnosticCheck {
    let version = env!("CARGO_PKG_VERSION");
    DiagnosticCheck {
        name: "sw binary".into(),
        status: CheckStatus::Pass,
        message: version.into(),
    }
}

fn check_native_ports() -> DiagnosticCheck {
    match ports_native::try_listening_ports() {
        None => DiagnosticCheck {
            name: "native port lookup".into(),
            status: CheckStatus::Warn,
            message: "unavailable on this platform; relies on lsof fallback".into(),
        },
        Some(Ok(_)) => DiagnosticCheck {
            name: "native port lookup".into(),
            status: CheckStatus::Pass,
            message: native_port_message(),
        },
        Some(Err(e)) => DiagnosticCheck {
            name: "native port lookup".into(),
            status: CheckStatus::Warn,
            message: format!("native lookup failed ({e}); lsof fallback may be used"),
        },
    }
}

fn native_port_message() -> String {
    #[cfg(target_os = "linux")]
    {
        "/proc/net/tcp".into()
    }
    #[cfg(target_os = "macos")]
    {
        "libproc".into()
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        "available".into()
    }
}

fn check_lsof() -> DiagnosticCheck {
    let found = Command::new("lsof").arg("-v").output().is_ok() || which::which("lsof").is_ok();
    if found {
        DiagnosticCheck {
            name: "lsof fallback".into(),
            status: CheckStatus::Pass,
            message: "on PATH".into(),
        }
    } else {
        DiagnosticCheck {
            name: "lsof fallback".into(),
            status: CheckStatus::Warn,
            message: "not found (fallback unavailable if native lookup fails)".into(),
        }
    }
}

fn check_history_path() -> DiagnosticCheck {
    match history_path() {
        Ok(path) => path_access_check("history writable", &path, true),
        Err(e) => DiagnosticCheck {
            name: "history writable".into(),
            status: CheckStatus::Fail,
            message: format!("cannot resolve history path: {e}"),
        },
    }
}

fn check_protect_config() -> DiagnosticCheck {
    let path = protect_config_path();
    if !path.exists() {
        return DiagnosticCheck {
            name: "protect config readable".into(),
            status: CheckStatus::Pass,
            message: format!("{} (optional file not present)", display_path(&path)),
        };
    }
    path_access_check("protect config readable", &path, false)
}

fn path_access_check(name: &str, path: &Path, needs_write: bool) -> DiagnosticCheck {
    let display = display_path(path);
    if needs_write {
        if let Some(parent) = path.parent() {
            if std::fs::create_dir_all(parent).is_err() {
                return DiagnosticCheck {
                    name: name.into(),
                    status: CheckStatus::Fail,
                    message: format!("cannot create parent directory for {display}"),
                };
            }
        }
        match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            Ok(_) => DiagnosticCheck {
                name: name.into(),
                status: CheckStatus::Pass,
                message: display,
            },
            Err(e) => DiagnosticCheck {
                name: name.into(),
                status: CheckStatus::Fail,
                message: format!("{display} not writable ({e})"),
            },
        }
    } else {
        match std::fs::read_to_string(path) {
            Ok(_) => DiagnosticCheck {
                name: name.into(),
                status: CheckStatus::Pass,
                message: display,
            },
            Err(e) => DiagnosticCheck {
                name: name.into(),
                status: CheckStatus::Fail,
                message: format!("{display} not readable ({e})"),
            },
        }
    }
}

fn check_orb_cli() -> DiagnosticCheck {
    cli_warn_check("orb CLI", "orb")
}

fn check_docker_cli() -> DiagnosticCheck {
    cli_warn_check("docker CLI", "docker")
}

fn cli_warn_check(name: &str, bin: &str) -> DiagnosticCheck {
    let reachable =
        Command::new(bin).arg("--version").output().is_ok() || which::which(bin).is_ok();
    if reachable {
        DiagnosticCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: "reachable".into(),
        }
    } else {
        DiagnosticCheck {
            name: name.into(),
            status: CheckStatus::Warn,
            message: format!("{bin} not on PATH (OrbStack/Docker features limited)"),
        }
    }
}

fn check_tty_color() -> DiagnosticCheck {
    let tty = atty::is(atty::Stream::Stdout);
    if tty {
        DiagnosticCheck {
            name: "stdout TTY".into(),
            status: CheckStatus::Pass,
            message: "terminal detected; colors enabled unless NO_COLOR is set".into(),
        }
    } else {
        DiagnosticCheck {
            name: "stdout TTY".into(),
            status: CheckStatus::Warn,
            message: "stdout is not a terminal; colors disabled for scripting".into(),
        }
    }
}

fn display_path(path: &Path) -> String {
    if let Ok(home) = std::env::var("HOME") {
        let home_path = Path::new(&home);
        if let Ok(stripped) = path.strip_prefix(home_path) {
            return format!("~/{}", stripped.display());
        }
    }
    path.display().to_string()
}

mod which {
    use std::process::Command;

    pub fn which(bin: &str) -> Result<(), ()> {
        let found = Command::new("which")
            .arg(bin)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
            || Command::new("command")
                .args(["-v", bin])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
        if found {
            Ok(())
        } else {
            Err(())
        }
    }
}

mod atty {
    pub enum Stream {
        Stdout,
    }

    pub fn is(stream: Stream) -> bool {
        match stream {
            Stream::Stdout => std::io::IsTerminal::is_terminal(&std::io::stdout()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_checks_non_empty() {
        let checks = collect_checks();
        assert!(checks.len() >= 7);
        assert!(checks.iter().any(|c| c.name == "sw binary"));
        assert!(checks.iter().any(|c| c.name == "native port lookup"));
    }

    #[test]
    fn binary_check_passes() {
        let check = check_binary();
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(!check.message.is_empty());
    }
}
