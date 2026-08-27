use std::collections::BTreeMap;
use std::path::Path;

use crate::process::ProcessInfo;

pub mod metadata;
pub mod session;
pub mod workspace;

use metadata::{enrich_project_path, infer_dev_script};
use session::infer_session_label;
use workspace::infer_workspace_package;

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectGroup {
    pub name: String,
    pub path: String,
    pub processes: Vec<ProcessInfo>,
    pub session_label: Option<String>,
    pub workspace_root: Option<String>,
    pub package_name: Option<String>,
    pub git_branch: Option<String>,
    pub compose_project: Option<String>,
    pub dev_script: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSummary {
    pub process_count: usize,
    pub memory_bytes: u64,
    pub ports: Vec<u16>,
}

pub fn summarize_group(group: &ProjectGroup) -> ProjectSummary {
    let mut ports: Vec<u16> = group
        .processes
        .iter()
        .flat_map(|p| p.ports.iter().copied())
        .collect();
    ports.sort_unstable();
    ports.dedup();
    ProjectSummary {
        process_count: group.processes.len(),
        memory_bytes: group.processes.iter().map(|p| p.memory_bytes).sum(),
        ports,
    }
}

/// Infer (project_name, project_path) from cwd or command line.
pub fn infer_project(proc: &ProcessInfo) -> Option<(String, String)> {
    if let Some(cwd) = proc.cwd.as_deref() {
        if let Some(ws) = infer_workspace_package(cwd) {
            return Some((
                ws.display_name,
                ws.package_path.to_string_lossy().into_owned(),
            ));
        }
        if let Some(pair) = from_path(cwd) {
            return Some(pair);
        }
    }
    if let Some(cmd) = proc.command.as_deref() {
        return from_command(cmd);
    }
    None
}

pub fn infer_project_metadata(proc: &ProcessInfo) -> ProjectMetadata {
    let mut meta = ProjectMetadata::default();
    if let Some(cwd) = proc.cwd.as_deref() {
        if let Some(ws) = infer_workspace_package(cwd) {
            meta.workspace_root = Some(ws.workspace_root.to_string_lossy().into_owned());
            meta.package_name = ws.package_name;
        }
    }
    meta
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectMetadata {
    pub workspace_root: Option<String>,
    pub package_name: Option<String>,
    pub session_label: Option<String>,
}

fn from_path(path: &str) -> Option<(String, String)> {
    if path.is_empty() || is_systemish_path(path) {
        return None;
    }
    let p = Path::new(path);
    let name = p.file_name()?.to_string_lossy();
    if name.is_empty() || name == "/" {
        return None;
    }
    let skip = [
        "tmp",
        "temp",
        "home",
        "Users",
        "var",
        "private",
        "proc",
        "bin",
        "sbin",
        "lib",
        "lib64",
        "libexec",
        "etc",
        "opt",
        "run",
        "dev",
        "node_modules",
        "target",
    ];
    if skip.iter().any(|s| name.eq_ignore_ascii_case(s)) {
        return None;
    }
    Some((name.into_owned(), path.to_string()))
}

fn is_systemish_path(path: &str) -> bool {
    // macOS per-user temp lives under /var/folders; TempDir and tools use it.
    // Treat it as user space so project inference still works in tests and temp cwds.
    if path.starts_with("/var/folders/") {
        return false;
    }
    let prefixes = [
        "/bin",
        "/sbin",
        "/usr",
        "/lib",
        "/lib64",
        "/System",
        "/proc",
        "/run",
        "/var",
        "/dev",
        "/etc",
        "/snap",
        "/boot",
        "/sys",
        "/srv",
        "/mnt",
        "/media",
        "/root",
        "/opt/cursor",
        "/home/ubuntu/.cursor-server",
        "/home/ubuntu/.cursor",
    ];
    prefixes
        .iter()
        .any(|p| path == *p || path.starts_with(&format!("{p}/")))
}

fn from_command(cmd: &str) -> Option<(String, String)> {
    // Prefer path segments before node_modules / target / .next
    for token in cmd.split_whitespace() {
        if !token.starts_with('/') && !token.starts_with('.') {
            continue;
        }
        let path = Path::new(token);
        let mut cur = path.to_path_buf();
        // Walk up looking for a project-ish directory
        while let Some(parent) = cur.parent() {
            let file = cur.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if matches!(
                file,
                "node_modules" | "target" | ".next" | "dist" | "build" | ".git"
            ) {
                if let Some(proj) = parent.to_str() {
                    return from_path(proj);
                }
            }
            if parent == Path::new("/") || parent.as_os_str().is_empty() {
                break;
            }
            cur = parent.to_path_buf();
        }
        // Fall back: parent of the binary/script if nested deeply
        if let Some(parent) = path.parent().and_then(|p| p.to_str()) {
            if parent.contains('/') {
                if let Some(pair) = from_path(parent) {
                    return Some(pair);
                }
            }
        }
    }
    None
}

pub fn group_projects(procs: &[ProcessInfo]) -> Vec<ProjectGroup> {
    let by_pid: std::collections::HashMap<u32, &ProcessInfo> =
        procs.iter().map(|p| (p.pid, p)).collect();
    let mut map: BTreeMap<String, ProjectGroup> = BTreeMap::new();
    for p in procs {
        let Some((name, path)) = infer_project(p) else {
            continue;
        };
        let meta = infer_project_metadata(p);
        let session = infer_session_label(p, &by_pid);
        let entry = map.entry(path.clone()).or_insert_with(|| {
            let (git_branch, compose_project) = enrich_project_path(&path);
            ProjectGroup {
                name: name.clone(),
                path: path.clone(),
                processes: Vec::new(),
                session_label: session.clone(),
                workspace_root: meta.workspace_root.clone(),
                package_name: meta.package_name.clone(),
                git_branch,
                compose_project,
                dev_script: None,
            }
        });
        if entry.session_label.is_none() {
            entry.session_label = session;
        }
        if entry.workspace_root.is_none() {
            entry.workspace_root = meta.workspace_root;
        }
        if entry.package_name.is_none() {
            entry.package_name = meta.package_name;
        }
        entry.merge_dev_script(p.command.as_deref());
        entry.processes.push(p.clone());
    }
    let mut groups: Vec<_> = map.into_values().collect();
    groups.sort_by_key(|a| a.name.to_lowercase());
    groups
}

impl ProjectGroup {
    fn merge_dev_script(&mut self, command: Option<&str>) {
        if self.dev_script.is_none() {
            self.dev_script = infer_dev_script(command);
        }
    }
}

pub fn find_projects_by_name<'a>(groups: &'a [ProjectGroup], query: &str) -> Vec<&'a ProjectGroup> {
    let q = query.to_lowercase();
    groups
        .iter()
        .filter(|g| g.name.to_lowercase().contains(&q) || g.path.to_lowercase().contains(&q))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proc(pid: u32, name: &str, cwd: Option<&str>, command: Option<&str>) -> ProcessInfo {
        ProcessInfo {
            pid,
            ppid: 1,
            name: name.into(),
            cpu: 0.0,
            memory_bytes: 10 * 1024 * 1024,
            ports: vec![],
            command: command.map(str::to_string),
            cwd: cwd.map(str::to_string),
            run_time_secs: 0,
            is_zombie: false,
        }
    }

    #[test]
    fn infers_from_cwd() {
        let p = proc(1, "node", Some("/Users/me/dev/my-app"), None);
        let (name, path) = infer_project(&p).unwrap();
        assert_eq!(name, "my-app");
        assert_eq!(path, "/Users/me/dev/my-app");
    }

    #[test]
    fn infers_from_node_modules_command() {
        let p = proc(
            2,
            "node",
            None,
            Some("/Users/me/dev/web/node_modules/.bin/vite"),
        );
        let (name, path) = infer_project(&p).unwrap();
        assert_eq!(name, "web");
        assert_eq!(path, "/Users/me/dev/web");
    }

    #[test]
    fn groups_by_path() {
        let procs = vec![
            proc(1, "node", Some("/Users/me/proj-a"), None),
            proc(2, "vite", Some("/Users/me/proj-a"), None),
            proc(3, "node", Some("/Users/me/proj-b"), None),
        ];
        let groups = group_projects(&procs);
        assert_eq!(groups.len(), 2);
        let a = groups.iter().find(|g| g.name == "proj-a").unwrap();
        assert_eq!(a.processes.len(), 2);
    }

    #[test]
    fn skips_system_paths() {
        let p = proc(1, "bash", Some("/usr/bin"), None);
        assert!(infer_project(&p).is_none());
        let p = proc(2, "node", Some("/home/ubuntu/.cursor-server/bin"), None);
        assert!(infer_project(&p).is_none());
        let p = proc(3, "snap", Some("/snap/firefox/current"), None);
        assert!(infer_project(&p).is_none());
        let p = proc(4, "node", Some("/var/log/my-app"), None);
        assert!(infer_project(&p).is_none());
    }

    #[test]
    fn allows_macos_var_folders_temp_cwd() {
        let p = proc(
            1,
            "node",
            Some("/var/folders/xx/yyyy/T/sweeper-proj-abc/my-app"),
            None,
        );
        let (name, path) = infer_project(&p).unwrap();
        assert_eq!(name, "my-app");
        assert_eq!(path, "/var/folders/xx/yyyy/T/sweeper-proj-abc/my-app");
    }

    #[test]
    fn find_by_name_substring() {
        let procs = vec![
            proc(1, "node", Some("/Users/me/my-app"), None),
            proc(2, "node", Some("/Users/me/other"), None),
        ];
        let groups = group_projects(&procs);
        let hits = find_projects_by_name(&groups, "my-app");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].name, "my-app");
    }

    #[test]
    fn summarize_group_totals() {
        let mut p1 = proc(1, "node", Some("/Users/me/app"), None);
        p1.memory_bytes = 100;
        p1.ports = vec![3000];
        let mut p2 = proc(2, "vite", Some("/Users/me/app"), None);
        p2.memory_bytes = 200;
        let group = ProjectGroup {
            name: "app".into(),
            path: "/Users/me/app".into(),
            processes: vec![p1, p2],
            session_label: None,
            workspace_root: None,
            package_name: None,
            git_branch: None,
            compose_project: None,
            dev_script: None,
        };
        let s = summarize_group(&group);
        assert_eq!(s.process_count, 2);
        assert_eq!(s.memory_bytes, 300);
        assert_eq!(s.ports, vec![3000]);
    }
}
