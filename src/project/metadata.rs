use std::fs;
use std::path::Path;

const COMPOSE_FILES: &[&str] = &[
    "docker-compose.yml",
    "docker-compose.yaml",
    "compose.yml",
    "compose.yaml",
];

/// Read git branch from `.git/HEAD` without spawning `git`.
pub fn infer_git_branch(project_path: &str) -> Option<String> {
    let head = Path::new(project_path).join(".git/HEAD");
    let content = fs::read_to_string(head).ok()?;
    let line = content.trim();
    if let Some(rest) = line.strip_prefix("ref: refs/heads/") {
        let branch = rest.trim();
        if branch.is_empty() {
            return None;
        }
        return Some(branch.to_string());
    }
    // Detached HEAD — show short commit prefix
    if line.len() >= 7 && line.chars().all(|c| c.is_ascii_hexdigit()) {
        return Some(format!("@{}", &line[..7]));
    }
    None
}

/// Detect docker compose project name from compose file `name:` field or directory.
pub fn infer_compose_project(project_path: &str) -> Option<String> {
    let root = Path::new(project_path);
    for file in COMPOSE_FILES {
        let path = root.join(file);
        if !path.is_file() {
            continue;
        }
        if let Ok(text) = fs::read_to_string(&path) {
            if let Some(name) = parse_compose_name(&text) {
                return Some(name);
            }
        }
        return root.file_name().map(|n| n.to_string_lossy().into_owned());
    }
    None
}

fn parse_compose_name(yaml: &str) -> Option<String> {
    for line in yaml.lines().take(30) {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix("name:") {
            let name = rest.trim().trim_matches('"').trim_matches('\'');
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    None
}

/// Infer dev script label from command line (e.g. `pnpm dev`, `npm run start`).
pub fn infer_dev_script(command: Option<&str>) -> Option<String> {
    let cmd = command?;
    let lower = cmd.to_lowercase();
    const PATTERNS: &[(&str, &str)] = &[
        ("pnpm dev", "pnpm dev"),
        ("pnpm run dev", "pnpm dev"),
        ("npm run dev", "npm dev"),
        ("npm run start", "npm start"),
        ("yarn dev", "yarn dev"),
        ("yarn start", "yarn start"),
        ("bun dev", "bun dev"),
        ("bun run dev", "bun dev"),
        ("vite dev", "vite dev"),
        ("next dev", "next dev"),
        ("astro dev", "astro dev"),
        ("turbo run dev", "turbo dev"),
        ("cargo run", "cargo run"),
        ("uvicorn", "uvicorn"),
        ("gunicorn", "gunicorn"),
        ("fastapi", "fastapi"),
    ];
    for (needle, label) in PATTERNS {
        if lower.contains(needle) {
            return Some(label.to_string());
        }
    }
    None
}

pub fn enrich_project_path(path: &str) -> (Option<String>, Option<String>) {
    (infer_git_branch(path), infer_compose_project(path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parses_git_branch_from_head() {
        let dir = tempfile::tempdir().unwrap();
        let git = dir.path().join(".git");
        fs::create_dir(&git).unwrap();
        fs::write(git.join("HEAD"), "ref: refs/heads/main\n").unwrap();
        assert_eq!(
            infer_git_branch(dir.path().to_str().unwrap()),
            Some("main".into())
        );
    }

    #[test]
    fn parses_compose_name_field() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("docker-compose.yml"),
            "name: my-stack\nservices:\n  web:\n    image: nginx\n",
        )
        .unwrap();
        assert_eq!(
            infer_compose_project(dir.path().to_str().unwrap()),
            Some("my-stack".into())
        );
    }

    #[test]
    fn infers_pnpm_dev_script() {
        assert_eq!(
            infer_dev_script(Some("node /app/node_modules/.bin/pnpm dev")),
            Some("pnpm dev".into())
        );
    }

    #[test]
    fn infers_next_dev() {
        assert_eq!(
            infer_dev_script(Some("node node_modules/.bin/next dev")),
            Some("next dev".into())
        );
    }
}
