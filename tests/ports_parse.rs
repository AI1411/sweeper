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
fn ignores_non_listen() {
    let line =
        "node 48291 user 20u IPv4 0x0 0t0 TCP 127.0.0.1:3000->127.0.0.1:4000 (ESTABLISHED)";
    assert!(parse_lsof_listen_line(line).is_none());
}
