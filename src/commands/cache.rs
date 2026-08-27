use crate::cache::{collect_caches_scanned, format_cache_report};
use crate::json_output::{emit_json, CacheJson};

pub fn run_cache(json: bool) -> anyhow::Result<()> {
    let scan = collect_caches_scanned();
    if json {
        return emit_json(&CacheJson::from_scan(&scan));
    }
    print!("{}", format_cache_report(&scan.entries));
    Ok(())
}
