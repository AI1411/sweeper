use std::fs;
use std::path::{Path, PathBuf};

const MAX_WALK: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceInfo {
    pub workspace_root: PathBuf,
    pub package_name: Option<String>,
    pub display_name: String,
    pub package_path: PathBuf,
}

/// Detect monorepo workspace root markers and optional package directory.
pub fn infer_workspace_package(cwd: &str) -> Option<WorkspaceInfo> {
    if cwd.is_empty() {
        return None;
    }
    let start = Path::new(cwd);
    if !start.is_absolute() {
        return None;
    }
    let mut cur = start.to_path_buf();
    for _ in 0..MAX_WALK {
        if is_workspace_root(&cur) {
            let workspace_root = cur.clone();
            let root_name = workspace_display_name(&workspace_root);
            let package_path = start.to_path_buf();
            let rel = package_path
                .strip_prefix(&workspace_root)
                .ok()
                .and_then(|p| p.to_str())
                .filter(|s| !s.is_empty())
                .map(str::to_string);
            let display_name = match rel.as_deref() {
                Some(sub) if sub != "." => format!("{root_name}/{sub}"),
                _ => root_name.clone(),
            };
            let package_name = rel.filter(|s| s != "." && !s.is_empty());
            return Some(WorkspaceInfo {
                workspace_root,
                package_name,
                display_name,
                package_path,
            });
        }
        if !cur.pop() {
            break;
        }
    }
    None
}

fn is_workspace_root(path: &Path) -> bool {
    if path.join("pnpm-workspace.yaml").is_file() {
        return true;
    }
    if path.join("turbo.json").is_file() || path.join("nx.json").is_file() {
        return true;
    }
    if let Ok(text) = fs::read_to_string(path.join("package.json")) {
        if text.contains("\"workspaces\"") {
            return true;
        }
    }
    false
}

fn workspace_display_name(root: &Path) -> String {
    if let Ok(text) = fs::read_to_string(root.join("package.json")) {
        if let Some(name) = parse_json_name_field(&text) {
            return name;
        }
    }
    root.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("workspace")
        .to_string()
}

fn parse_json_name_field(text: &str) -> Option<String> {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
        return value
            .get("name")
            .and_then(|v| v.as_str())
            .map(str::to_string);
    }
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("\"name\"") {
            let rest = rest.trim_start_matches(':').trim();
            if let Some(quoted) = rest.strip_prefix('"').and_then(|s| s.split('"').next()) {
                if !quoted.is_empty() {
                    return Some(quoted.to_string());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }

    #[test]
    fn detects_pnpm_workspace_package() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        write(&root.join("pnpm-workspace.yaml"), "packages:\n  - apps/*\n");
        write(&root.join("package.json"), r#"{"name":"my-monorepo"}"#);
        let app = root.join("apps/web");
        write(&app.join("package.json"), r#"{"name":"web"}"#);
        let info = infer_workspace_package(app.to_str().unwrap()).unwrap();
        assert_eq!(info.display_name, "my-monorepo/apps/web");
        assert_eq!(
            info.workspace_root
                .canonicalize()
                .unwrap_or_else(|_| root.to_path_buf()),
            root.canonicalize().unwrap_or_else(|_| root.to_path_buf())
        );
        assert_eq!(info.package_name.as_deref(), Some("apps/web"));
    }

    #[test]
    fn detects_npm_workspaces_root() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        write(
            &root.join("package.json"),
            r#"{"name":"repo","workspaces":["packages/*"]}"#,
        );
        let pkg = root.join("packages/api");
        fs::create_dir_all(&pkg).unwrap();
        let info = infer_workspace_package(pkg.to_str().unwrap()).unwrap();
        assert_eq!(info.display_name, "repo/packages/api");
    }

    #[test]
    fn non_workspace_returns_none() {
        let tmp = TempDir::new().unwrap();
        let app = tmp.path().join("standalone-app");
        fs::create_dir_all(&app).unwrap();
        assert!(infer_workspace_package(app.to_str().unwrap()).is_none());
    }
}
