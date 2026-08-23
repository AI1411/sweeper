use sweeper::process::ports::parse_lsof_listen_line;

#[test]
fn parses_typical_lsof_line() {
    // COMMAND PID USER FD TYPE DEVICE SIZE/OFF NODE NAME
    let line = "node 48291 user 20u IPv4 0x0 0t0 TCP *:3000 (LISTEN)";
    let (pid, port) = parse_lsof_listen_line(line).expect("parse");
    assert_eq!(pid, 48291);
    assert_eq!(port, 3000);
}

#[test]
fn parses_ipv4_bind_address() {
    let line = "python3 2340 user 3u IPv4 0x0 0t0 TCP 127.0.0.1:8080 (LISTEN)";
    let (pid, port) = parse_lsof_listen_line(line).expect("parse");
    assert_eq!(pid, 2340);
    assert_eq!(port, 8080);
}

#[test]
fn ignores_non_listen() {
    let line = "node 48291 user 20u IPv4 0x0 0t0 TCP 127.0.0.1:3000->127.0.0.1:4000 (ESTABLISHED)";
    assert!(parse_lsof_listen_line(line).is_none());
}

#[test]
fn ignores_short_line() {
    assert!(parse_lsof_listen_line("node 1").is_none());
}

#[test]
fn ignores_header_like_line() {
    let line = "COMMAND PID USER FD TYPE DEVICE SIZE/OFF NODE NAME";
    assert!(parse_lsof_listen_line(line).is_none());
}

#[test]
fn parses_proc_net_tcp_listen_line() {
    use sweeper::process::ports_native::parse_proc_net_tcp_line;
    let line = "  0: 0100007F:1F90 00000000:0000 0A 00000000:00000000 00:00000000 00000000    1000        0 12345 1 00000000 100 0 0 10 0";
    let (port, inode) = parse_proc_net_tcp_line(line).expect("parse");
    assert_eq!(port, 8080);
    assert_eq!(inode, 12345);
}

#[test]
fn ignores_proc_net_tcp_non_listen() {
    use sweeper::process::ports_native::parse_proc_net_tcp_line;
    let line = "  1: 0100007F:1F90 0100007F:3E8 01 00000000:00000000 00:00000000 00000000    1000        0 12346 1 00000000 20 4 30 10 -1";
    assert!(parse_proc_net_tcp_line(line).is_none());
}

#[test]
fn listening_ports_native_on_linux() {
    sweeper::process::ports::listening_ports().expect("ports");
}
