//! Terminal styling helpers. Honors `NO_COLOR` and non-TTY stdout via owo-colors.

use std::fmt::Display;

use owo_colors::{OwoColorize, Stream::Stdout, Style};

use crate::process::kill::KillOutcome;

fn paint(s: impl Display, style: Style) -> String {
    format!("{}", s.if_supports_color(Stdout, |t| t.style(style)))
}

pub fn header(s: impl Display) -> String {
    paint(s, Style::new().cyan().bold())
}

pub fn dim(s: impl Display) -> String {
    paint(s, Style::new().dimmed())
}

pub fn process_name(s: impl Display) -> String {
    paint(s, Style::new().bold())
}

pub fn pid(s: impl Display) -> String {
    paint(s, Style::new().bright_black())
}

pub fn port(s: impl Display) -> String {
    paint(s, Style::new().yellow().bold())
}

pub fn cpu(pct: f32) -> String {
    let text = format!("{pct:.1}%");
    if pct >= 50.0 {
        paint(text, Style::new().red().bold())
    } else if pct >= 20.0 {
        paint(text, Style::new().yellow())
    } else {
        paint(text, Style::new().green())
    }
}

pub fn mem(s: impl Display) -> String {
    paint(s, Style::new().magenta())
}

pub fn success(s: impl Display) -> String {
    paint(s, Style::new().green().bold())
}

pub fn warn(s: impl Display) -> String {
    paint(s, Style::new().yellow())
}

pub fn error(s: impl Display) -> String {
    paint(s, Style::new().red().bold())
}

pub fn rank(n: usize) -> String {
    paint(format!("{n}."), Style::new().dimmed())
}

pub fn kill_outcome(outcome: KillOutcome) -> String {
    let text = format!("{outcome:?}");
    match outcome {
        KillOutcome::Terminated | KillOutcome::ForceKilled => success(text),
        KillOutcome::StillAlive => error(text),
        KillOutcome::SkippedProtected => warn(text),
    }
}
