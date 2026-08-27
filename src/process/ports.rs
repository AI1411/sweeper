use std::process::Command;
use std::sync::{Mutex, Once, OnceLock};
use std::time::{Duration, Instant};

use std::collections::HashMap;

use crate::error::{Result, SweeperError};
use crate::process::ports_native;
use crate::process::ProcessInfo;

static LSOF_FALLBACK_HINT: Once = Once::new();
const PORT_CACHE_TTL: Duration = Duration::from_secs(3);

struct PortCacheEntry {
    expires: Instant,
    pairs: Vec<(u16, u32)>,
}

static PORT_CACHE: OnceLock<Mutex<Option<PortCacheEntry>>> = OnceLock::new();

pub fn clear_port_cache() {
    if let Some(lock) = PORT_CACHE.get() {
        *lock.lock().expect("port cache lock") = None;
    }
}

fn hint_lsof_fallback() {
    LSOF_FALLBACK_HINT.call_once(|| {
        eprintln!("note: native port lookup unavailable; falling back to lsof");
    });
}

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

fn listening_ports_lsof() -> Result<Vec<(u16, u32)>> {
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

fn pids_for_port_lsof(port: u16) -> Result<Vec<u32>> {
    let output = Command::new("lsof")
        .args(["-nP", &format!("-iTCP:{port}"), "-sTCP:LISTEN", "-t"])
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

fn fetch_listening_ports() -> Result<Vec<(u16, u32)>> {
    match ports_native::try_listening_ports() {
        Some(result) => match result {
            Ok(pairs) => Ok(pairs),
            Err(_) => {
                hint_lsof_fallback();
                listening_ports_lsof()
            }
        },
        None => {
            hint_lsof_fallback();
            listening_ports_lsof()
        }
    }
}

pub fn listening_ports() -> Result<Vec<(u16, u32)>> {
    listening_ports_cached(false)
}

pub fn listening_ports_cached(bypass_cache: bool) -> Result<Vec<(u16, u32)>> {
    if bypass_cache {
        return fetch_listening_ports();
    }
    let lock = PORT_CACHE.get_or_init(|| Mutex::new(None));
    let mut guard = lock.lock().expect("port cache lock");
    if let Some(entry) = guard.as_ref() {
        if Instant::now() < entry.expires {
            return Ok(entry.pairs.clone());
        }
    }
    let pairs = fetch_listening_ports()?;
    *guard = Some(PortCacheEntry {
        expires: Instant::now() + PORT_CACHE_TTL,
        pairs: pairs.clone(),
    });
    Ok(pairs)
}

/// Resolve PIDs listening on `port`. Single-port callers may use targeted native/lsof lookup.
pub fn pids_for_port(port: u16) -> Result<Vec<u32>> {
    match ports_native::try_pids_for_port(port) {
        Some(result) => match result {
            Ok(pids) => Ok(pids),
            Err(_) => {
                hint_lsof_fallback();
                pids_for_port_lsof(port)
            }
        },
        None => {
            hint_lsof_fallback();
            pids_for_port_lsof(port)
        }
    }
}

/// Resolve multiple ports using the cached LISTEN table when possible.
pub fn pids_for_ports(ports: &[u16]) -> Result<HashMap<u16, Vec<u32>>> {
    if ports.is_empty() {
        return Ok(HashMap::new());
    }
    if ports.len() == 1 {
        let port = ports[0];
        return Ok([(port, pids_for_port(port)?)].into_iter().collect());
    }
    let listen = listening_ports_cached(false)?;
    let mut out: HashMap<u16, Vec<u32>> = HashMap::new();
    for &port in ports {
        let pids: Vec<u32> = listen
            .iter()
            .filter(|(p, _)| *p == port)
            .map(|(_, pid)| *pid)
            .collect();
        out.insert(port, pids);
    }
    Ok(out)
}

pub fn merge_ports(procs: &mut [ProcessInfo], port_map: &[(u16, u32)]) {
    if port_map.is_empty() {
        return;
    }
    let mut by_pid: HashMap<u32, &mut ProcessInfo> = HashMap::with_capacity(procs.len());
    for proc in procs.iter_mut() {
        by_pid.insert(proc.pid, proc);
    }
    for (port, pid) in port_map {
        if let Some(p) = by_pid.get_mut(pid) {
            if !p.ports.contains(port) {
                p.ports.push(*port);
            }
        }
    }
}

#[cfg(test)]
mod cache_tests {
    use super::*;

    #[test]
    fn clear_port_cache_is_safe() {
        clear_port_cache();
        let first = listening_ports_cached(false).expect("ports");
        let second = listening_ports_cached(false).expect("ports again");
        assert_eq!(first, second);
        clear_port_cache();
    }

    #[test]
    fn pids_for_ports_empty() {
        assert!(pids_for_ports(&[]).expect("empty").is_empty());
    }

    #[test]
    fn pids_for_ports_single_delegates() {
        let map = pids_for_ports(&[1]).expect("single");
        assert_eq!(map.len(), 1);
        assert!(map.contains_key(&1));
    }
}
