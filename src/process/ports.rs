use std::process::Command;

use crate::error::{Result, SweeperError};
use crate::process::ProcessInfo;

/// Parse one `lsof -nP -iTCP -sTCP:LISTEN` style line → (pid, port)
pub fn parse_lsof_listen_line(line: &str) -> Option<(u32, u16)> {
    if !line.contains("(LISTEN)") {
        return None;
    }
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 9 {
        return None;
    }
    let pid: u32 = parts[1].parse().ok()?;
    let name_field = parts
        .iter()
        .rev()
        .find(|p| p.contains(':') && !p.contains("->"))?;
    let port_str = name_field.rsplit(':').next()?;
    let port: u16 = port_str.parse().ok()?;
    Some((pid, port))
}

pub fn listening_ports() -> Result<Vec<(u16, u32)>> {
    let output = Command::new("lsof")
        .args(["-nP", "-iTCP", "-sTCP:LISTEN"])
        .output()
        .map_err(|e| SweeperError::Lsof(e.to_string()))?;
    if !output.status.success() && output.stdout.is_empty() {
        return Err(SweeperError::Lsof(format!(
            "exit {}",
            output.status.code().unwrap_or(-1)
        )));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut pairs = Vec::new();
    for line in text.lines().skip(1) {
        if let Some((pid, port)) = parse_lsof_listen_line(line) {
            pairs.push((port, pid));
        }
    }
    Ok(pairs)
}

pub fn pids_for_port(port: u16) -> Result<Vec<u32>> {
    let output = Command::new("lsof")
        .args(["-nP", &format!("-iTCP:{}", port), "-sTCP:LISTEN", "-t"])
        .output()
        .map_err(|e| SweeperError::Lsof(e.to_string()))?;
    let text = String::from_utf8_lossy(&output.stdout);
    let mut pids = Vec::new();
    for line in text.lines() {
        if let Ok(pid) = line.trim().parse::<u32>() {
            pids.push(pid);
        }
    }
    Ok(pids)
}

pub fn merge_ports(procs: &mut [ProcessInfo], port_map: &[(u16, u32)]) {
    for (port, pid) in port_map {
        if let Some(p) = procs.iter_mut().find(|p| p.pid == *pid) {
            if !p.ports.contains(port) {
                p.ports.push(*port);
            }
        }
    }
}
