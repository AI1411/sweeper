use std::io::{self, Write};

use crate::style;

pub fn confirm(prompt: &str) -> io::Result<bool> {
    print!("{} {} ", prompt, style::dim("[y/N]"));
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(matches!(buf.trim(), "y" | "Y" | "yes" | "YES"))
}
