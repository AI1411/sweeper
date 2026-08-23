use std::ffi::c_void;
use std::mem::MaybeUninit;

use crate::error::{Result, SweeperError};

const PROC_ALL_PIDS: u32 = 1;
const PROC_PIDLISTFDS: i32 = 1;
const PROC_PIDFDSOCKETINFO: i32 = 3;
const PROX_FDTYPE_SOCKET: u32 = 2;
const SOCKINFO_TCP: i32 = 2;
const TCP_STATE_LISTEN: i32 = 1;

#[repr(C)]
#[derive(Clone, Copy)]
struct ProcFdInfo {
    proc_fd: i32,
    proc_fdtype: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct InSockInfo {
    insi_fport: i32,
    insi_lport: i32,
    insi_gencnt: u64,
    insi_flags: u32,
    insi_flow: u32,
    insi_vflag: u8,
    insi_ip_ttl: u8,
    rfu_1: u16,
    insi_faddr: [u8; 16],
    insi_laddr: [u8; 16],
    insi_v4: u8,
    insi_v6: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct TcpSockInfo {
    tcpsi_ini: InSockInfo,
    tcpsi_state: i32,
    tcpsi_timer: [i32; 4],
    tcpsi_mss: i32,
    tcpsi_flags: u32,
    rfu_1: u32,
    tcpsi_tp: u64,
}

#[repr(C)]
struct SocketInfo {
    soi_stat: u64,
    soi_so: u64,
    soi_proto: u32,
    soi_family: u32,
    soi_type: u32,
    soi_protocol: u32,
    rfu_1: u32,
    soi_kind: i32,
    rfu_2: u32,
    soi_proto_union: TcpProtoUnion,
}

#[repr(C)]
union TcpProtoUnion {
    pri_tcp: TcpSockInfo,
}

#[repr(C)]
struct SocketFdInfo {
    psi: SocketInfo,
}

unsafe extern "C" {
    fn proc_listpids(type_: u32, typeinfo: u32, buffer: *mut c_void, buffersize: i32) -> i32;
    fn proc_pidinfo(pid: i32, flavor: i32, arg: u64, buffer: *mut c_void, buffersize: i32) -> i32;
    fn proc_pidfdinfo(pid: i32, fd: i32, flavor: i32, buffer: *mut c_void, buffersize: i32) -> i32;
}

fn list_pids() -> Result<Vec<i32>> {
    let mut buf = vec![0u8; 4096];
    let nbytes =
        unsafe { proc_listpids(PROC_ALL_PIDS, 0, buf.as_mut_ptr().cast(), buf.len() as i32) };
    if nbytes <= 0 {
        return Err(SweeperError::Lsof("proc_listpids failed".into()));
    }
    let count = nbytes as usize / std::mem::size_of::<i32>();
    let mut pids = Vec::new();
    for i in 0..count {
        let pid = i32::from_ne_bytes(buf[i * 4..i * 4 + 4].try_into().unwrap());
        if pid > 0 {
            pids.push(pid);
        }
    }
    Ok(pids)
}

fn socket_listen_ports(pid: i32) -> Vec<u16> {
    let fd_size = std::mem::size_of::<ProcFdInfo>() as i32;
    let mut stack_fds = [ProcFdInfo {
        proc_fd: 0,
        proc_fdtype: 0,
    }; 64];
    let nbytes = unsafe {
        proc_pidinfo(
            pid,
            PROC_PIDLISTFDS,
            0,
            stack_fds.as_mut_ptr().cast(),
            fd_size * 64,
        )
    };
    if nbytes <= 0 {
        return Vec::new();
    }
    let count = nbytes as usize / std::mem::size_of::<ProcFdInfo>();
    let mut ports = Vec::new();
    for fdinfo in &stack_fds[..count] {
        if fdinfo.proc_fdtype != PROX_FDTYPE_SOCKET {
            continue;
        }
        let mut si: MaybeUninit<SocketFdInfo> = MaybeUninit::uninit();
        let got = unsafe {
            proc_pidfdinfo(
                pid,
                fdinfo.proc_fd,
                PROC_PIDFDSOCKETINFO,
                si.as_mut_ptr().cast(),
                std::mem::size_of::<SocketFdInfo>() as i32,
            )
        };
        if got <= 0 {
            continue;
        }
        let si = unsafe { si.assume_init() };
        if si.psi.soi_kind != SOCKINFO_TCP {
            continue;
        }
        let tcp = unsafe { si.psi.soi_proto_union.pri_tcp };
        if tcp.tcpsi_state != TCP_STATE_LISTEN {
            continue;
        }
        let port = tcp.tcpsi_ini.insi_lport as u16;
        if port != 0 {
            ports.push(port);
        }
    }
    ports
}

pub fn listening_ports() -> Result<Vec<(u16, u32)>> {
    let pids = list_pids()?;
    let mut pairs = Vec::new();
    for pid in pids {
        for port in socket_listen_ports(pid) {
            pairs.push((port, pid as u32));
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
