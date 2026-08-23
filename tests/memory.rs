use sweeper::commands::memory::report_from;
use sweeper::memory::{format_bytes, MemorySort, SystemMemorySnapshot};

const STATS: &str = "\
postgres\t1.2GiB / 7.776GiB
redis\t420MiB / 7.776GiB
api\t850MiB / 7.776GiB
";

const PS: &str = "\
postgres\tUp 2 hours
redis\tUp 2 hours
api\tUp 2 hours
";

fn system() -> SystemMemorySnapshot {
    SystemMemorySnapshot {
        total_bytes: 128 * 1024 * 1024 * 1024,
        used_bytes: 42 * 1024 * 1024 * 1024,
        available_bytes: 86 * 1024 * 1024 * 1024,
    }
}

#[test]
fn memory_report_sorts_containers_by_memory() {
    let report = report_from(
        system(),
        Some(18_400_000_000),
        STATS,
        PS,
        MemorySort::Memory,
    )
    .unwrap();
    assert_eq!(report.containers[0].name, "postgres");
    assert!(report.unattributed_bytes.unwrap() > 15_000_000_000);
}

#[test]
fn memory_report_sorts_by_name() {
    let report = report_from(system(), Some(18_400_000_000), STATS, PS, MemorySort::Name).unwrap();
    assert_eq!(report.containers[0].name, "api");
}

#[test]
fn format_bytes_matches_fixture_scale() {
    assert!(
        format_bytes(1_200_000_000).contains("GB") || format_bytes(1_200_000_000).contains("MB")
    );
}
