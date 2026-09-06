use sweeper::commands::select::{
    format_process_header, format_process_row, parse_selection, ParseHigh,
};
use sweeper::process::ProcessInfo;

fn proc(pid: u32, name: &str, ports: Vec<u16>, cwd: Option<&str>) -> ProcessInfo {
    ProcessInfo {
        pid,
        ppid: 1,
        name: name.into(),
        cpu: 2.1,
        memory_bytes: 184 * 1024 * 1024,
        ports,
        command: None,
        cwd: cwd.map(str::to_string),
        run_time_secs: 60,
        is_zombie: false,
    }
}

#[test]
fn parse_comma_indices() {
    assert_eq!(parse_selection("1,3", 3, ParseHigh::None), Some(vec![0, 2]));
}

#[test]
fn parse_range_and_mix() {
    assert_eq!(
        parse_selection("1-3,5", 5, ParseHigh::None),
        Some(vec![0, 1, 2, 4])
    );
}

#[test]
fn parse_whitespace_separated() {
    assert_eq!(parse_selection("1 3", 3, ParseHigh::None), Some(vec![0, 2]));
}

#[test]
fn parse_all_keywords() {
    for input in ["all", "a", "*"] {
        assert_eq!(
            parse_selection(input, 4, ParseHigh::None),
            Some(vec![0, 1, 2, 3]),
            "input={input}"
        );
    }
}

#[test]
fn parse_high_uses_provided_indices() {
    assert_eq!(
        parse_selection("high", 5, ParseHigh::Indices(&[0, 3])),
        Some(vec![0, 3])
    );
    assert_eq!(
        parse_selection("h", 5, ParseHigh::Indices(&[1])),
        Some(vec![1])
    );
}

#[test]
fn parse_high_without_list_is_invalid() {
    assert_eq!(parse_selection("high", 3, ParseHigh::None), None);
}

#[test]
fn parse_cancel() {
    for input in ["q", "quit", "n", "no", ""] {
        assert_eq!(
            parse_selection(input, 3, ParseHigh::None),
            Some(vec![]),
            "input={input:?}"
        );
    }
}

#[test]
fn parse_invalid() {
    assert_eq!(parse_selection("0,1", 2, ParseHigh::None), None);
    assert_eq!(parse_selection("99", 2, ParseHigh::None), None);
    assert_eq!(parse_selection("3-1", 3, ParseHigh::None), None);
}

#[test]
fn identity_row_includes_port_and_project() {
    std::env::set_var("NO_COLOR", "1");
    let p = proc(4812, "node", vec![3000], Some("/Users/dev/my-app"));
    let row = format_process_row(1, &p);
    assert!(row.contains("4812"), "{row}");
    assert!(row.contains("node"), "{row}");
    assert!(row.contains(":3000"), "{row}");
    assert!(row.contains("my-app"), "{row}");
    let header = format_process_header();
    assert!(header.contains("PID"), "{header}");
    assert!(header.contains("PORT"), "{header}");
    assert!(header.contains("PROJECT"), "{header}");
}
