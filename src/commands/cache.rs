use crate::cache::{collect_caches, format_cache_report};
use crate::json_output::{emit_json, CacheJson};

pub fn run_cache(json: bool) -> anyhow::Result<()> {
    let entries = collect_caches();
    if json {
        return emit_json(&CacheJson::from_entries(&entries));
    }
    print!("{}", format_cache_report(&entries));
    Ok(())
}
