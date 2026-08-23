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
    assert_eq!(all[0].signal, KillSignal::Term);
}

#[test]
fn missing_file_loads_empty() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("missing.json");
    let all = load_entries_at(&path).unwrap();
    assert!(all.is_empty());
}

#[test]
fn empty_file_loads_empty() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("history.json");
    std::fs::write(&path, "   ").unwrap();
    let all = load_entries_at(&path).unwrap();
    assert!(all.is_empty());
}

#[test]
fn caps_at_two_hundred_entries() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("history.json");
    for i in 0..205 {
        let e = HistoryEntry::new(i, "node", vec![], KillSignal::Term, "ok");
        append_entry_at(&path, e).unwrap();
    }
    let all = load_entries_at(&path).unwrap();
    assert_eq!(all.len(), 200);
    assert_eq!(all.first().unwrap().pid, 5);
    assert_eq!(all.last().unwrap().pid, 204);
}

#[test]
fn preserves_kill_signal() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("history.json");
    let e = HistoryEntry::new(1, "node", vec![1, 2], KillSignal::Kill, "force");
    append_entry_at(&path, e).unwrap();
    let all = load_entries_at(&path).unwrap();
    assert_eq!(all[0].signal, KillSignal::Kill);
    assert_eq!(all[0].ports, vec![1, 2]);
}

#[test]
#[cfg(target_os = "linux")]
fn history_path_uses_xdg_data_dir() {
    let path = sweeper::history::history_path().expect("history path");
    let s = path.to_string_lossy();
    assert!(s.contains(".local/share"));
    assert!(s.ends_with("history.json"));
}

#[test]
#[cfg(target_os = "linux")]
fn protect_config_uses_xdg_config_dir() {
    let path = sweeper::process::protect::protect_config_path();
    let s = path.to_string_lossy();
    assert!(s.contains(".config/sweeper"));
    assert!(s.ends_with("protect.toml"));
}
