use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

use sweeper::project::{group_projects, infer_project, session::infer_session_label};
use sweeper::ProcessInfo;

fn proc(pid: u32, ppid: u32, name: &str, cwd: Option<&str>) -> ProcessInfo {
    ProcessInfo {
        pid,
        ppid,
        name: name.into(),
        cpu: 0.0,
        memory_bytes: 0,
        ports: vec![],
        command: None,
        cwd: cwd.map(str::to_string),
        run_time_secs: 0,
        is_zombie: false,
    }
}

#[test]
fn groups_monorepo_packages_separately() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("apps/web")).unwrap();
    fs::create_dir_all(root.join("apps/api")).unwrap();
    fs::write(root.join("pnpm-workspace.yaml"), "packages:\n  - apps/*\n").unwrap();
    fs::write(root.join("package.json"), r#"{"name":"my-monorepo"}"#).unwrap();

    let web = root.join("apps/web");
    let api = root.join("apps/api");
    let procs = vec![
        proc(1, 10, "vite", Some(web.to_str().unwrap())),
        proc(2, 10, "node", Some(web.to_str().unwrap())),
        proc(3, 10, "bun", Some(api.to_str().unwrap())),
    ];
    let groups = group_projects(&procs);
    assert_eq!(groups.len(), 2);
    let names: Vec<_> = groups.iter().map(|g| g.name.as_str()).collect();
    assert!(names.iter().any(|n| n.contains("web")));
    assert!(names.iter().any(|n| n.contains("api")));
    assert!(groups
        .iter()
        .all(|g| g.workspace_root.is_some() && g.package_name.is_some()));
}

#[test]
fn worktree_paths_are_distinct_groups() {
    let procs = vec![
        proc(1, 10, "node", Some("/Users/dev/my-app-worktree-a")),
        proc(2, 10, "node", Some("/Users/dev/my-app-worktree-b")),
    ];
    let groups = group_projects(&procs);
    assert_eq!(groups.len(), 2);
}

#[test]
fn tmux_session_label_on_group() {
    let tmux = proc(10, 1, "tmux", None);
    let child = proc(100, 10, "node", Some("/Users/dev/my-monorepo/apps/web"));
    let by_pid: HashMap<u32, &ProcessInfo> = [(10, &tmux)].into_iter().collect();
    assert_eq!(
        infer_session_label(&child, &by_pid).as_deref(),
        Some("tmux")
    );

    let groups = group_projects(&[tmux, child]);
    let group = groups
        .iter()
        .find(|g| g.processes.iter().any(|p| p.pid == 100))
        .expect("web group");
    assert_eq!(group.session_label.as_deref(), Some("tmux"));
}

#[test]
fn project_group_includes_git_branch_and_dev_script() {
    let tmp = TempDir::new().unwrap();
    // Use an explicit project directory name so basename-based inference is stable
    // even when the TempDir itself sits under macOS /var/folders.
    let root = tmp.path().join("my-app");
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::write(root.join(".git/HEAD"), "ref: refs/heads/feature-x\n").unwrap();
    let mut procs = vec![proc(1, 10, "node", Some(root.to_str().unwrap()))];
    procs[0].command = Some("pnpm dev".into());
    let groups = group_projects(&procs);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].name, "my-app");
    assert_eq!(groups[0].git_branch.as_deref(), Some("feature-x"));
    assert_eq!(groups[0].dev_script.as_deref(), Some("pnpm dev"));
}

#[test]
fn infer_project_uses_workspace_display_name() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("apps/web")).unwrap();
    fs::write(root.join("turbo.json"), "{}").unwrap();
    fs::write(root.join("package.json"), r#"{"name":"mono"}"#).unwrap();
    let cwd = root.join("apps/web");
    let p = proc(1, 1, "node", Some(cwd.to_str().unwrap()));
    let (name, path) = infer_project(&p).unwrap();
    assert!(name.contains("mono"));
    assert!(name.contains("apps/web"));
    assert_eq!(path, cwd.to_string_lossy());
}
