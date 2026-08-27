use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct SweeperConfig {
    #[serde(default)]
    pub clean: CleanConfig,
    #[serde(default)]
    pub tui: TuiConfig,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct CleanConfig {
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub default_high_only: bool,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct TuiConfig {
    #[serde(default)]
    pub auto_refresh_secs: Option<u64>,
    #[serde(default)]
    pub default_view: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ConfigLoadResult {
    pub config: SweeperConfig,
    pub path: PathBuf,
    pub parse_error: Option<String>,
}

pub fn config_path() -> PathBuf {
    let dirs = ProjectDirs::from("com", "sweeper", "sweeper").expect("home directory");
    dirs.config_dir().join("config.toml")
}

pub fn parse_config_text(text: &str) -> Result<SweeperConfig, toml::de::Error> {
    toml::from_str(text)
}

pub fn load_config_from_path(path: &Path) -> ConfigLoadResult {
    if !path.exists() {
        return ConfigLoadResult {
            config: SweeperConfig::default(),
            path: path.to_path_buf(),
            parse_error: None,
        };
    }
    match std::fs::read_to_string(path) {
        Ok(text) => match parse_config_text(&text) {
            Ok(config) => ConfigLoadResult {
                config,
                path: path.to_path_buf(),
                parse_error: None,
            },
            Err(e) => ConfigLoadResult {
                config: SweeperConfig::default(),
                path: path.to_path_buf(),
                parse_error: Some(e.to_string()),
            },
        },
        Err(e) => ConfigLoadResult {
            config: SweeperConfig::default(),
            path: path.to_path_buf(),
            parse_error: Some(e.to_string()),
        },
    }
}

static CONFIG: std::sync::OnceLock<ConfigLoadResult> = std::sync::OnceLock::new();

pub fn load_config() -> &'static ConfigLoadResult {
    CONFIG.get_or_init(|| load_config_from_path(&config_path()))
}

pub fn effective_clean_excludes(cli_excludes: &[String]) -> Vec<String> {
    let mut excludes = load_config().config.clean.exclude.clone();
    excludes.extend(crate::clean::excludes_from_env());
    excludes.extend(cli_excludes.iter().cloned());
    excludes
}

pub fn effective_clean_high_only() -> bool {
    if matches!(
        std::env::var("SWEEPER_CLEAN_HIGH_ONLY").ok().as_deref(),
        Some("1") | Some("true") | Some("yes")
    ) {
        return true;
    }
    load_config().config.clean.default_high_only
}

pub fn effective_tui_refresh_secs() -> u64 {
    std::env::var("SWEEPER_TUI_REFRESH_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .filter(|&n| n > 0)
        .or(load_config().config.tui.auto_refresh_secs)
        .filter(|&n| n > 0)
        .unwrap_or(2)
}

pub fn effective_tui_default_view() -> Option<String> {
    std::env::var("SWEEPER_TUI_DEFAULT_VIEW")
        .ok()
        .or_else(|| load_config().config.tui.default_view.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parses_clean_and_tui_sections() {
        let text = r#"
[clean]
exclude = ["postgres", "redis"]
default_high_only = true

[tui]
auto_refresh_secs = 3
default_view = "projects"
"#;
        let cfg = parse_config_text(text).expect("parse");
        assert_eq!(cfg.clean.exclude, vec!["postgres", "redis"]);
        assert!(cfg.clean.default_high_only);
        assert_eq!(cfg.tui.auto_refresh_secs, Some(3));
        assert_eq!(cfg.tui.default_view.as_deref(), Some("projects"));
    }

    #[test]
    fn invalid_toml_returns_error() {
        assert!(parse_config_text("[clean\nexclude = bad").is_err());
    }

    #[test]
    fn load_from_temp_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let mut file = std::fs::File::create(&path).expect("create");
        writeln!(file, "[tui]\nauto_refresh_secs = 5\n").expect("write");
        let loaded = load_config_from_path(&path);
        assert!(loaded.parse_error.is_none());
        assert_eq!(loaded.config.tui.auto_refresh_secs, Some(5));
    }

    #[test]
    fn invalid_file_falls_back_with_parse_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "not = [valid").expect("write");
        let loaded = load_config_from_path(&path);
        assert!(loaded.parse_error.is_some());
        assert_eq!(loaded.config, SweeperConfig::default());
    }
}
