#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::parse_proc_net_tcp_line;
#[cfg(target_os = "macos")]
mod macos;

use crate::error::Result;

/// Native LISTEN port discovery. Returns `None` when unavailable on this platform.
pub fn try_listening_ports() -> Option<Result<Vec<(u16, u32)>>> {
    #[cfg(target_os = "linux")]
    {
        Some(linux::listening_ports())
    }
    #[cfg(target_os = "macos")]
    {
        Some(macos::listening_ports())
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        None
    }
}

pub fn try_pids_for_port(port: u16) -> Option<Result<Vec<u32>>> {
    #[cfg(target_os = "linux")]
    {
        Some(linux::pids_for_port(port))
    }
    #[cfg(target_os = "macos")]
    {
        Some(macos::pids_for_port(port))
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        None
    }
}
