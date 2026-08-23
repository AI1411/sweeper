use std::thread;
use std::time::Duration;

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use super::format_bytes;
use super::{collect_memory_report, orbstack_vm_bytes, MemoryReport};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchSample {
    pub timestamp: OffsetDateTime,
    pub vm_bytes: Option<u64>,
    pub containers: Vec<(String, u64)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchDelta {
    pub vm_delta_bytes: Option<i64>,
    pub elapsed_secs: u64,
}

pub fn compute_delta(prev: &WatchSample, curr: &WatchSample) -> WatchDelta {
    let elapsed_secs = (curr.timestamp - prev.timestamp)
        .whole_seconds()
        .unsigned_abs();
    let vm_delta_bytes = match (prev.vm_bytes, curr.vm_bytes) {
        (Some(a), Some(b)) => Some(b as i64 - a as i64),
        _ => None,
    };
    WatchDelta {
        vm_delta_bytes,
        elapsed_secs,
    }
}

pub fn format_delta_line(delta: &WatchDelta) -> Option<String> {
    let bytes = delta.vm_delta_bytes?;
    if bytes == 0 || delta.elapsed_secs == 0 {
        return None;
    }
    let sign = if bytes > 0 { "↑" } else { "↓" };
    let abs = bytes.unsigned_abs();
    Some(format!(
        "  {sign} {} / {} sec",
        format_bytes(abs),
        delta.elapsed_secs
    ))
}

pub fn sample_from_report(report: &MemoryReport) -> WatchSample {
    WatchSample {
        timestamp: OffsetDateTime::now_local().unwrap_or(OffsetDateTime::UNIX_EPOCH),
        vm_bytes: report.orbstack_vm_bytes,
        containers: report
            .containers
            .iter()
            .map(|c| (c.name.clone(), c.memory_bytes))
            .collect(),
    }
}

pub fn collect_watch_sample() -> WatchSample {
    if let Ok(report) = collect_memory_report() {
        return sample_from_report(&report);
    }
    WatchSample {
        timestamp: OffsetDateTime::now_local().unwrap_or(OffsetDateTime::UNIX_EPOCH),
        vm_bytes: orbstack_vm_bytes(),
        containers: vec![],
    }
}

pub fn run_memory_watch(interval_secs: u64, containers: bool, json: bool) -> anyhow::Result<()> {
    let interval = Duration::from_secs(interval_secs.max(1));
    let mut prev: Option<WatchSample> = None;
    loop {
        let sample = collect_watch_sample();
        if json {
            emit_watch_json(&sample, prev.as_ref())?;
        } else {
            print_watch_line(&sample, prev.as_ref(), containers);
        }
        prev = Some(sample);
        thread::sleep(interval);
    }
}

fn print_watch_line(sample: &WatchSample, prev: Option<&WatchSample>, containers: bool) {
    let time = sample
        .timestamp
        .format(&Rfc3339)
        .unwrap_or_else(|_| sample.timestamp.to_string());
    let short_time = time.chars().take(19).collect::<String>();
    if let Some(vm) = sample.vm_bytes {
        print!("{short_time}     {}", format_bytes(vm));
        if let Some(p) = prev {
            let delta = compute_delta(p, sample);
            if let Some(hint) = format_delta_line(&delta) {
                print!("{hint}");
            }
        }
        println!();
    } else {
        println!("{short_time}     OrbStack VM not detected");
    }
    if containers {
        for (name, bytes) in &sample.containers {
            let mut line = format!("  {name:<16} {}", format_bytes(*bytes));
            if let Some(p) = prev {
                let prev_bytes = p
                    .containers
                    .iter()
                    .find(|(n, _)| n == name)
                    .map(|(_, b)| *b);
                if let (Some(a), Some(b)) = (prev_bytes, Some(*bytes)) {
                    let delta = WatchDelta {
                        vm_delta_bytes: Some(b as i64 - a as i64),
                        elapsed_secs: (sample.timestamp - p.timestamp)
                            .whole_seconds()
                            .unsigned_abs(),
                    };
                    if let Some(hint) = format_delta_line(&delta) {
                        line.push_str(&hint);
                    }
                }
            }
            println!("{line}");
        }
    }
}

fn emit_watch_json(sample: &WatchSample, prev: Option<&WatchSample>) -> anyhow::Result<()> {
    let delta = prev.map(|p| compute_delta(p, sample));
    let line = serde_json::json!({
        "timestamp": sample.timestamp.format(&Rfc3339).ok(),
        "vm_bytes": sample.vm_bytes,
        "containers": sample.containers.iter().map(|(n,b)| serde_json::json!({"name": n, "memory_bytes": b})).collect::<Vec<_>>(),
        "vm_delta_bytes": delta.as_ref().and_then(|d| d.vm_delta_bytes),
        "elapsed_secs": delta.as_ref().map(|d| d.elapsed_secs),
    });
    println!("{}", serde_json::to_string(&line)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delta_positive_when_vm_grows() {
        let t0 = OffsetDateTime::UNIX_EPOCH;
        let t1 = t0 + time::Duration::seconds(15);
        let prev = WatchSample {
            timestamp: t0,
            vm_bytes: Some(1_000_000_000),
            containers: vec![],
        };
        let curr = WatchSample {
            timestamp: t1,
            vm_bytes: Some(2_100_000_000),
            containers: vec![],
        };
        let delta = compute_delta(&prev, &curr);
        assert_eq!(delta.vm_delta_bytes, Some(1_100_000_000));
        assert_eq!(delta.elapsed_secs, 15);
        let hint = format_delta_line(&delta).unwrap();
        assert!(hint.contains('↑'));
    }
}
