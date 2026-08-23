use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::error::{Result, SweeperError};

const TCP_LISTEN: &str = "0A";

/// Parse one `/proc/net/tcp` or `/proc/net/tcp6` line → (port, socket inode).
pub fn parse_proc_net_tcp_line(line: &str) -> Option<(u16, u64)> {
    let line = line.trim();
    if line.is_empty() || line.starts_with("sl") {
        return None;
    }
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 10 {
        return None;
    }
    if parts[3] != TCP_LISTEN {
        return None;
    }
    let local = parts[1];
    let (_, port_hex) = local.rsplit_once(':')?;
    let port = u16::from_str_radix(port_hex, 16).ok()?;
    let inode = parts[9].parse::<u64>().ok()?;
    Some((port, inode))
}

fn read_listen_entries(path: &Path) -> Result<Vec<(u16, u64)>> {
    let text = fs::read_to_string(path).map_err(SweeperError::Io)?;
    let mut out = Vec::new();
    for line in text.lines().skip(1) {
        if let Some(entry) = parse_proc_net_tcp_line(line) {
            out.push(entry);
        }
    }
    Ok(out)
}

fn inode_pid_map() -> Result<HashMap<u64, u32>> {
    let mut map = HashMap::new();
    let proc = fs::read_dir("/proc").map_err(SweeperError::Io)?;
    for entry in proc {
        let entry = entry.map_err(SweeperError::Io)?;
        let pid = entry.file_name().to_string_lossy().parse::<u32>().ok();
        if pid.is_none() {
            continue;
        }
        let pid = pid.unwrap();
        let fd_dir = format!("/proc/{}/fd", pid);
        let fds = match fs::read_dir(&fd_dir) {
            Ok(fds) => fds,
            Err(_) => continue,
        };
        for fd in fds {
            let fd = fd.map_err(SweeperError::Io)?;
            let link = match fs::read_link(fd.path()) {
                Ok(link) => link,
                Err(_) => continue,
            };
            let link = link.to_string_lossy();
            if let Some(rest) = link.strip_prefix("socket:[") {
                if let Some(inode_str) = rest.strip_suffix(']') {
                    if let Ok(inode) = inode_str.parse::<u64>() {
                        map.insert(inode, pid);
                    }
                }
            }
        }
    }
    Ok(map)
}

pub fn listening_ports() -> Result<Vec<(u16, u32)>> {
    let mut entries = read_listen_entries(Path::new("/proc/net/tcp"))?;
    if let Ok(tcp6) = read_listen_entries(Path::new("/proc/net/tcp6")) {
        entries.extend(tcp6);
    }
    let inode_map = inode_pid_map()?;
    let mut pairs = Vec::new();
    for (port, inode) in entries {
        if let Some(pid) = inode_map.get(&inode) {
            pairs.push((port, *pid));
        }
    }
    Ok(pairs)
}

pub fn pids_for_port(port: u16) -> Result<Vec<u32>> {
    let pairs = listening_ports()?;
    let mut pids: Vec<u32> = pairs
        .into_iter()
        .filter(|(p, _)| *p == port)
        .map(|(_, pid)| pid)
        .collect();
    pids.sort_unstable();
    pids.dedup();
    Ok(pids)
}
