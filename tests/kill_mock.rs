use sweeper::history::{append_entry_at, load_entries_at, HistoryEntry, KillSignal};
use sweeper::process::kill::{kill_pid, set_kill_hook, KillOutcome};
use sweeper::report::{format_kill_summary, KillResult};
use tempfile::tempdir;

#[test]
fn mock_kill_records_history_and_summary() {
    set_kill_hook(Some(Box::new(|pid, name, force| {
        assert_eq!(pid, 9001);
        assert_eq!(name, "node");
        assert!(!force);
        Ok(KillOutcome::Terminated)
    })));

    let outcome = kill_pid(9001, "node", false).expect("mock kill");
    assert_eq!(outcome, KillOutcome::Terminated);

    let dir = tempdir().expect("tempdir");
    let history_path = dir.path().join("history.json");
    append_entry_at(
        &history_path,
        HistoryEntry::new(9001, "node", vec![3000], KillSignal::Term, "Terminated"),
    )
    .expect("append history");

    let entries = load_entries_at(&history_path).expect("load history");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].pid, 9001);
    assert_eq!(entries[0].ports, vec![3000]);

    let summary = format_kill_summary(&[KillResult::new(
        64 * 1024 * 1024,
        vec![3000],
        KillOutcome::Terminated,
    )]);
    assert!(summary.contains("Terminated 1 process(es)"));
    assert!(summary.contains(":3000"));

    set_kill_hook(None);
}
