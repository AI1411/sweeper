use sweeper::commands::port::merge_port_bindings;

#[test]
fn dedupes_pid_across_ports() {
    let map = merge_port_bindings(&[(3000, 10), (3001, 10), (5173, 20)]);
    assert_eq!(map[&10], vec![3000, 3001]);
    assert_eq!(map[&20], vec![5173]);
    assert_eq!(map.len(), 2);
}

#[test]
fn keeps_single_pid_single_port() {
    let map = merge_port_bindings(&[(8080, 42)]);
    assert_eq!(map[&42], vec![8080]);
}
