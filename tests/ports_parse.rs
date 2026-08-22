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
