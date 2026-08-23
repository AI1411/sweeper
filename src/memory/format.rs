/// Human-readable byte size (binary GB/MB).
pub fn format_bytes(bytes: u64) -> String {
    const GB: u64 = 1024 * 1024 * 1024;
    const MB: u64 = 1024 * 1024;
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{} MB", bytes / MB)
    } else if bytes >= 1024 {
        format!("{} KB", bytes / 1024)
    } else {
        format!("{bytes} B")
    }
}

/// Estimated value prefix for heuristic reclaim figures.
pub fn format_estimate(bytes: u64) -> String {
    format!("~{}", format_bytes(bytes))
}

/// Parse Docker-style memory strings (`1.2GiB`, `420MiB`, `850KiB`).
pub fn parse_docker_bytes(input: &str) -> Option<u64> {
    let s = input.trim();
    if s.is_empty() {
        return None;
    }
    let (num_part, unit) = s
        .chars()
        .position(|c| !c.is_ascii_digit() && c != '.')
        .map(|idx| s.split_at(idx))
        .unwrap_or((s, ""));
    let value: f64 = num_part.parse().ok()?;
    let multiplier = match unit.to_ascii_lowercase().as_str() {
        "b" | "" => 1.0,
        "kib" | "kb" | "k" => 1024.0,
        "mib" | "mb" | "m" => 1024.0 * 1024.0,
        "gib" | "gb" | "g" => 1024.0 * 1024.0 * 1024.0,
        "tib" | "tb" | "t" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };
    Some((value * multiplier) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_gigabytes() {
        assert_eq!(format_bytes(18_400_000_000), "17.1 GB");
    }

    #[test]
    fn formats_megabytes() {
        assert_eq!(format_bytes(420 * 1024 * 1024), "420 MB");
    }

    #[test]
    fn parses_docker_units() {
        assert_eq!(parse_docker_bytes("1.2GiB"), Some(1_288_490_188));
        assert_eq!(parse_docker_bytes("420MiB"), Some(440_401_920));
        assert_eq!(parse_docker_bytes("850KiB"), Some(870_400));
    }
}
