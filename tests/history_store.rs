use sweeper::history::{append_entry_at, load_entries_at, HistoryEntry, KillSignal};
use tempfile::tempdir;

#[test]
fn append_and_load() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("history.json");
    let e = HistoryEntry::new(123, "node", vec![3000], KillSignal::Term, "terminated");
    append_entry_at(&path, e.clone()).unwrap();
    let all = load_entries_at(&path).unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].pid, 123);
}
