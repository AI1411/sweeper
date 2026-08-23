use sysinfo::System;

use super::SystemMemorySnapshot;

pub fn system_memory() -> SystemMemorySnapshot {
    let mut sys = System::new();
    sys.refresh_memory();
    let total = sys.total_memory();
    let available = sys.available_memory();
    let used = total.saturating_sub(available);
    SystemMemorySnapshot {
        total_bytes: total,
        used_bytes: used,
        available_bytes: available,
    }
}
