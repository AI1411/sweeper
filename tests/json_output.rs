use sweeper::json_output::{emit_json, PortRow};

#[test]
fn emit_json_ports_rows() {
    std::env::set_var("NO_COLOR", "1");
    let rows = vec![
        PortRow {
            port: 3000,
            pid: 100,
            process: "node".into(),
        },
        PortRow {
            port: 8080,
            pid: 200,
            process: "python3".into(),
        },
    ];
    let json = serde_json::to_string(&rows).expect("serialize");
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).expect("parse");
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0]["port"], 3000);
    assert_eq!(parsed[0]["pid"], 100);
    assert_eq!(parsed[0]["process"], "node");
}

#[test]
fn ports_list_json_integration() {
    let rows = vec![PortRow {
        port: 5173,
        pid: 42,
        process: "vite".into(),
    }];
    let json = serde_json::to_string_pretty(&rows).expect("pretty");
    assert!(json.contains("\"port\": 5173"));
    assert!(emit_json(&rows).is_ok());
}
