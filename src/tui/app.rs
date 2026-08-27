use std::collections::HashSet;

use ratatui::widgets::TableState;

use crate::clean::{confidence_level, propose_leftovers, CleanCandidate};
use crate::memory::{format_bytes, format_estimate};
use crate::process::tree::{layout_tree_rows, TreeRow};
use crate::process::ProcessInfo;
use crate::project::{group_projects, summarize_group, ProjectGroup};
use crate::tui::resources::{load_resource_snapshot, ResourcePanel, ResourceSnapshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Processes,
    Projects,
    Clean,
}

/// Kill parameters awaiting TUI confirmation ([y/N]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingKill {
    pub force: bool,
    pub tree: bool,
}

pub struct App {
    pub processes: Vec<ProcessInfo>,
    pub filtered: Vec<usize>,
    pub cursor: usize,
    pub selected: HashSet<u32>,
    pub query: String,
    pub searching: bool,
    pub should_quit: bool,
    pub status: String,
    /// When true, only show processes that have at least one LISTEN port.
    pub ports_only: bool,
    pub last_ports: Vec<(u16, u32)>,
    pub table_state: TableState,
    /// Visible table body rows (updated each draw for page navigation).
    pub viewport_rows: usize,
    /// When set, kill keys showed a preview and await `y` / `n` / Esc.
    pub confirming_kill: Option<PendingKill>,
    /// Process detail panel for the current row.
    pub show_detail: bool,
    pub tree_view: bool,
    pub tree_rows: Vec<TreeRow>,
    pub view_mode: ViewMode,
    pub project_groups: Vec<ProjectGroup>,
    pub expanded_project: Option<usize>,
    pub clean_proposals: Vec<CleanCandidate>,
    pub clean_filtered: Vec<usize>,
    pub clean_high_only: bool,
    pub resources_open: bool,
    pub resource_panel: ResourcePanel,
    pub resource_snapshot: ResourceSnapshot,
    pub confirming_reclaim: bool,
    pub show_help_overlay: bool,
}

impl App {
    pub fn new(processes: Vec<ProcessInfo>) -> Self {
        let mut app = Self {
            processes,
            filtered: Vec::new(),
            cursor: 0,
            selected: HashSet::new(),
            query: String::new(),
            searching: false,
            should_quit: false,
            status: "Loading listening ports…".into(),
            ports_only: false,
            last_ports: Vec::new(),
            table_state: TableState::default(),
            viewport_rows: 20,
            confirming_kill: None,
            show_detail: false,
            tree_view: false,
            tree_rows: Vec::new(),
            view_mode: ViewMode::Processes,
            project_groups: Vec::new(),
            expanded_project: None,
            clean_proposals: Vec::new(),
            clean_filtered: Vec::new(),
            clean_high_only: false,
            resources_open: false,
            resource_panel: ResourcePanel::default(),
            resource_snapshot: ResourceSnapshot::unavailable(),
            confirming_reclaim: false,
            show_help_overlay: false,
        };
        app.refilter();
        app
    }

    pub fn request_kill_confirm(&mut self, force: bool, tree: bool) {
        if self.pids_to_kill().is_empty() {
            self.status = "Nothing to kill".into();
            self.confirming_kill = None;
            return;
        }
        self.confirming_kill = Some(PendingKill { force, tree });
        self.status = format!("{} | Confirm kill? [y/N]", self.format_kill_preview(tree));
    }

    pub fn cancel_kill_confirm(&mut self) {
        self.confirming_kill = None;
        self.status = "Kill cancelled".into();
    }

    pub fn take_pending_kill(&mut self) -> Option<PendingKill> {
        self.confirming_kill.take()
    }

    pub fn is_confirming_kill(&self) -> bool {
        self.confirming_kill.is_some()
    }

    pub fn is_confirming_reclaim(&self) -> bool {
        self.confirming_reclaim
    }

    pub fn request_reclaim_confirm(&mut self) {
        if !self.resource_snapshot.available {
            self.status = "OrbStack/Docker not available".into();
            return;
        }
        let reclaim = self.resource_snapshot.memory_reclaimable.unwrap_or(0);
        if reclaim == 0 {
            self.status = "No reclaimable memory estimated".into();
            return;
        }
        self.confirming_reclaim = true;
        self.status = format!(
            "Reclaim approximately {}? [y/N]",
            crate::memory::format_estimate(reclaim)
        );
    }

    pub fn cancel_reclaim_confirm(&mut self) {
        self.confirming_reclaim = false;
        self.status = "Reclaim cancelled".into();
    }

    pub fn apply_resource_snapshot(&mut self, snapshot: ResourceSnapshot) {
        self.resource_snapshot = snapshot;
        if self.resources_open && !self.resource_snapshot.available {
            self.resources_open = false;
            self.status = "OrbStack/Docker not detected".into();
        }
    }

    pub fn toggle_resources_view(&mut self) {
        if self.resources_open {
            self.resources_open = false;
            self.resource_panel = ResourcePanel::Summary;
            self.confirming_reclaim = false;
            self.status = "View: processes".into();
            self.refilter();
            return;
        }
        self.resources_open = true;
        self.resource_panel = ResourcePanel::Summary;
        self.view_mode = ViewMode::Processes;
        self.show_detail = false;
        self.tree_view = false;
        self.expanded_project = None;
        self.cursor = 0;
        if !self.resource_snapshot.available {
            self.resource_snapshot = load_resource_snapshot();
        }
        if !self.resource_snapshot.available {
            self.resources_open = false;
            self.status = "OrbStack/Docker not detected".into();
            return;
        }
        self.status = "OrbStack [R] reclaim [C] containers [D] docker [Esc] back".into();
    }

    pub fn set_resource_panel(&mut self, panel: ResourcePanel) {
        if !self.resources_open {
            return;
        }
        self.resource_panel = panel;
        self.cursor = 0;
        match panel {
            ResourcePanel::Summary => {
                self.status = "OrbStack summary [R] reclaim [Esc] back".into();
            }
            ResourcePanel::Containers => {
                self.status = format!(
                    "{} container(s) [Esc] back",
                    self.resource_snapshot.containers.len()
                );
            }
            ResourcePanel::Docker => {
                self.status = "Docker disk overview [Esc] back".into();
            }
        }
    }

    pub fn resource_lines(&self) -> Vec<String> {
        let snap = &self.resource_snapshot;
        match self.resource_panel {
            ResourcePanel::Summary => {
                let mut lines = vec!["OrbStack Memory".into(), "────────────────".into()];
                if let Some(vm) = snap.orbstack_vm_bytes {
                    lines.push(format!("VM Memory        {}", format_bytes(vm)));
                }
                lines.push(format!(
                    "Containers       {}",
                    format_bytes(snap.container_total_bytes)
                ));
                if let Some(r) = snap.memory_reclaimable {
                    lines.push(format!("Reclaimable      {}", format_estimate(r)));
                }
                if let Some(est) = &snap.reclaim_estimate {
                    lines.push(String::new());
                    lines.push("Estimated sources:".into());
                    lines.push(format!(
                        "Linux page cache {}",
                        format_estimate(est.page_cache_bytes)
                    ));
                    lines.push(format!(
                        "Filesystem cache {}",
                        format_estimate(est.filesystem_cache_bytes)
                    ));
                }
                lines
            }
            ResourcePanel::Containers => {
                let mut lines = vec!["Containers".into(), "────────────────".into()];
                for c in &snap.containers {
                    lines.push(format!("{:<16} {}", c.name, format_bytes(c.memory_bytes)));
                }
                lines
            }
            ResourcePanel::Docker => {
                let mut lines = vec!["Docker Disk".into(), "────────────────".into()];
                if let Some(disk) = &snap.disk_report {
                    for row in &disk.rows {
                        lines.push(format!(
                            "{:<16} {}",
                            row.kind,
                            format_bytes(row.total_bytes)
                        ));
                    }
                    lines.push(format!(
                        "Reclaimable      {}",
                        format_bytes(disk.reclaimable_bytes)
                    ));
                }
                lines
            }
        }
    }

    pub fn toggle_project_view(&mut self) {
        if self.view_mode == ViewMode::Projects {
            self.view_mode = ViewMode::Processes;
            self.expanded_project = None;
            self.cursor = 0;
            self.refilter();
            self.status = "View: processes".into();
            return;
        }
        self.resources_open = false;
        self.view_mode = ViewMode::Projects;
        self.tree_view = false;
        self.tree_rows.clear();
        self.show_detail = false;
        self.expanded_project = None;
        self.clean_high_only = false;
        self.clean_proposals.clear();
        self.clean_filtered.clear();
        self.selected.clear();
        self.cursor = 0;
        self.project_groups = group_projects(&self.processes);
        self.refilter();
        self.status = format!("Projects: {} group(s) [P back]", self.project_groups.len());
    }

    pub fn toggle_clean_view(&mut self) {
        if self.view_mode == ViewMode::Clean {
            self.view_mode = ViewMode::Processes;
            self.clean_proposals.clear();
            self.clean_filtered.clear();
            self.clean_high_only = false;
            self.cursor = 0;
            self.refilter();
            self.status = "View: processes".into();
            return;
        }
        self.resources_open = false;
        self.view_mode = ViewMode::Clean;
        self.tree_view = false;
        self.tree_rows.clear();
        self.show_detail = false;
        self.expanded_project = None;
        self.selected.clear();
        self.cursor = 0;
        self.refresh_clean_proposals();
        self.status = format!(
            "Clean: {} candidate(s) [c back, H high-only]",
            self.clean_filtered.len()
        );
    }

    pub fn toggle_clean_high_only(&mut self) {
        if self.view_mode != ViewMode::Clean {
            return;
        }
        self.clean_high_only = !self.clean_high_only;
        self.cursor = 0;
        self.refilter_clean();
        let filter = if self.clean_high_only {
            "high confidence only"
        } else {
            "all confidence levels"
        };
        self.status = format!(
            "Clean: {} candidate(s) ({filter})",
            self.clean_filtered.len()
        );
    }

    pub fn refresh_clean_proposals(&mut self) {
        self.clean_proposals = propose_leftovers(&self.processes, &self.last_ports);
        self.refilter_clean();
    }

    fn refilter_clean(&mut self) {
        self.clean_filtered = self
            .clean_proposals
            .iter()
            .enumerate()
            .filter(|(_, c)| !self.clean_high_only || confidence_level(c) == "high")
            .map(|(i, _)| i)
            .collect();
        if self.cursor >= self.clean_filtered.len() && !self.clean_filtered.is_empty() {
            self.cursor = self.clean_filtered.len() - 1;
        }
        if self.clean_filtered.is_empty() {
            self.cursor = 0;
        }
        self.sync_table_state();
    }

    pub fn in_clean_list(&self) -> bool {
        self.view_mode == ViewMode::Clean
    }

    pub fn visible_clean_candidates(&self) -> Vec<&CleanCandidate> {
        self.clean_filtered
            .iter()
            .filter_map(|i| self.clean_proposals.get(*i))
            .collect()
    }

    pub fn toggle_project_expand(&mut self) {
        if self.view_mode != ViewMode::Projects {
            return;
        }
        if self.expanded_project.is_some() {
            self.collapse_project();
            return;
        }
        if let Some(idx) = self.project_group_index_at_cursor() {
            self.expanded_project = Some(idx);
            self.selected.clear();
            self.cursor = 0;
            self.refilter();
            if let Some(g) = self.project_groups.get(idx) {
                self.status = format!("Expanded {} (Enter/Esc collapse)", g.name);
            }
        }
    }

    pub fn collapse_project(&mut self) {
        if self.expanded_project.is_some() {
            self.expanded_project = None;
            self.selected.clear();
            self.cursor = 0;
            self.refilter();
            self.status = "Collapsed project".into();
        }
    }

    pub fn in_project_list(&self) -> bool {
        self.view_mode == ViewMode::Projects && self.expanded_project.is_none()
    }

    pub fn visible_project_groups(&self) -> Vec<&ProjectGroup> {
        let q = self.query.to_lowercase();
        self.project_groups
            .iter()
            .filter(|g| {
                q.is_empty()
                    || g.name.to_lowercase().contains(&q)
                    || g.path.to_lowercase().contains(&q)
            })
            .collect()
    }

    fn project_group_index_at_cursor(&self) -> Option<usize> {
        self.visible_project_groups()
            .get(self.cursor)
            .and_then(|g| self.project_groups.iter().position(|pg| pg.path == g.path))
    }

    pub fn current_project_group(&self) -> Option<&ProjectGroup> {
        if !self.in_project_list() {
            return None;
        }
        self.visible_project_groups().get(self.cursor).copied()
    }

    pub fn toggle_detail(&mut self) {
        if self.in_project_list() {
            return;
        }
        if self.current_process().is_some() {
            self.show_detail = !self.show_detail;
        } else {
            self.show_detail = false;
        }
    }

    pub fn toggle_tree_view(&mut self) {
        if self.view_mode != ViewMode::Processes {
            return;
        }
        self.tree_view = !self.tree_view;
        self.rebuild_tree_rows();
        self.cursor = 0;
        self.status = if self.tree_view {
            "View: process tree (e to flat list)".into()
        } else {
            "View: flat list".into()
        };
    }

    pub fn rebuild_tree_rows(&mut self) {
        if self.tree_view {
            self.tree_rows = layout_tree_rows(&self.processes, &self.filtered);
        } else {
            self.tree_rows.clear();
        }
    }

    fn display_len(&self) -> usize {
        if self.in_clean_list() {
            self.clean_filtered.len()
        } else if self.in_project_list() {
            self.visible_project_groups().len()
        } else if self.tree_view {
            self.tree_rows.len()
        } else {
            self.filtered.len()
        }
    }

    pub fn current_process(&self) -> Option<&ProcessInfo> {
        if self.in_clean_list() {
            return self
                .clean_filtered
                .get(self.cursor)
                .and_then(|i| self.clean_proposals.get(*i))
                .map(|c| &c.process);
        }
        if self.tree_view {
            self.tree_rows
                .get(self.cursor)
                .and_then(|row| self.processes.get(row.process_index))
        } else {
            self.filtered
                .get(self.cursor)
                .and_then(|i| self.processes.get(*i))
        }
    }

    /// Multi-line detail text for the current row.
    pub fn format_process_detail(&self) -> Vec<String> {
        let p = match self.current_process() {
            Some(p) => p,
            None => return vec!["No process selected".into()],
        };
        let ports = if p.ports.is_empty() {
            "-".into()
        } else {
            p.ports
                .iter()
                .map(|port| format!(":{port}"))
                .collect::<Vec<_>>()
                .join(",")
        };
        let cmd = p.command.as_deref().unwrap_or("-");
        let cwd = p.cwd.as_deref().unwrap_or("-");
        let project = crate::project::infer_project(p)
            .map(|(name, _)| name)
            .unwrap_or_else(|| "-".into());
        let age = crate::clean::format_age(p.run_time_secs);
        vec![
            format!(
                "{} pid {}  ppid {}  CPU {:.1}%  MEM {:.0} MB  Port {}  Started {}",
                p.name,
                p.pid,
                p.ppid,
                p.cpu,
                p.memory_mb(),
                ports,
                age
            ),
            format!("Command: {cmd}"),
            format!("CWD: {cwd}"),
            format!("Parent: {}", Self::parent_chain(p, &self.processes)),
            format!("Project: {project}"),
        ]
    }

    fn parent_chain(proc: &ProcessInfo, procs: &[ProcessInfo]) -> String {
        let mut ancestors = Vec::new();
        let mut ppid = proc.ppid;
        let mut depth = 0;
        while ppid != 0 && depth < 12 {
            if let Some(parent) = procs.iter().find(|x| x.pid == ppid) {
                ancestors.push(parent.name.clone());
                ppid = parent.ppid;
            } else {
                ancestors.push(format!("ppid {ppid}"));
                break;
            }
            depth += 1;
        }
        ancestors.reverse();
        ancestors.push(proc.name.clone());
        ancestors.join(" → ")
    }

    pub fn sync_table_state(&mut self) {
        if self.in_clean_list() || self.in_project_list() {
            let len = self.display_len();
            self.table_state
                .select(if len == 0 { None } else { Some(self.cursor) });
            return;
        }
        if self.display_len() == 0 {
            self.table_state.select(None);
        } else {
            self.table_state.select(Some(self.cursor));
        }
    }

    pub fn refilter(&mut self) {
        if self.view_mode == ViewMode::Clean {
            self.refresh_clean_proposals();
            return;
        }
        if self.view_mode == ViewMode::Projects {
            self.project_groups = group_projects(&self.processes);
            if let Some(idx) = self.expanded_project {
                if let Some(group) = self.project_groups.get(idx) {
                    self.filtered = group
                        .processes
                        .iter()
                        .filter_map(|m| self.processes.iter().position(|p| p.pid == m.pid))
                        .collect();
                } else {
                    self.expanded_project = None;
                    self.filtered.clear();
                }
            } else {
                self.filtered.clear();
                let len = self.visible_project_groups().len();
                if self.cursor >= len && len > 0 {
                    self.cursor = len - 1;
                }
                if len == 0 {
                    self.cursor = 0;
                }
            }
            if self.expanded_project.is_some() {
                if self.cursor >= self.filtered.len() && !self.filtered.is_empty() {
                    self.cursor = self.filtered.len() - 1;
                }
                if self.filtered.is_empty() {
                    self.cursor = 0;
                }
            }
            self.sync_table_state();
            return;
        }
        let q = self.query.to_lowercase();
        let q_port = q.trim_start_matches(':');
        self.filtered = self
            .processes
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                if self.ports_only && p.ports.is_empty() {
                    return false;
                }
                if q.is_empty() {
                    return true;
                }
                p.name.to_lowercase().contains(&q)
                    || p.ports.iter().any(|port| {
                        let s = port.to_string();
                        s.contains(q_port) || format!(":{port}").contains(&q)
                    })
                    || p.command
                        .as_ref()
                        .map(|c| c.to_lowercase().contains(&q))
                        .unwrap_or(false)
            })
            .map(|(i, _)| i)
            .collect();
        if self.cursor >= self.filtered.len() && !self.filtered.is_empty() {
            self.cursor = self.filtered.len() - 1;
        }
        if self.filtered.is_empty() {
            self.cursor = 0;
        }
        self.rebuild_tree_rows();
        if self.tree_view && self.cursor >= self.tree_rows.len() && !self.tree_rows.is_empty() {
            self.cursor = self.tree_rows.len() - 1;
        }
        if self.tree_view && self.tree_rows.is_empty() {
            self.cursor = 0;
        }
        self.sync_table_state();
    }

    pub fn current_pid(&self) -> Option<u32> {
        self.current_process().map(|p| p.pid)
    }

    pub fn move_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.sync_table_state();
        }
    }

    pub fn move_down(&mut self) {
        let len = self.display_len();
        if len > 0 && self.cursor + 1 < len {
            self.cursor += 1;
            self.sync_table_state();
        }
    }

    pub fn move_first(&mut self) {
        if self.display_len() > 0 {
            self.cursor = 0;
            self.sync_table_state();
        }
    }

    pub fn move_last(&mut self) {
        let len = self.display_len();
        if len > 0 {
            self.cursor = len - 1;
            self.sync_table_state();
        }
    }

    pub fn move_page_up(&mut self, step: usize) {
        if self.display_len() == 0 {
            return;
        }
        let step = step.max(1);
        self.cursor = self.cursor.saturating_sub(step);
        self.sync_table_state();
    }

    pub fn move_page_down(&mut self, step: usize) {
        let len = self.display_len();
        if len == 0 {
            return;
        }
        let step = step.max(1);
        self.cursor = (self.cursor + step).min(len - 1);
        self.sync_table_state();
    }

    pub fn set_viewport_rows(&mut self, rows: usize) {
        self.viewport_rows = rows.max(1);
    }

    /// One-line kill preview for status bar (current row or multi-select).
    pub fn format_kill_preview(&self, tree: bool) -> String {
        let roots = self.pids_to_kill();
        if roots.is_empty() {
            return "Nothing to kill".into();
        }
        let tree_hint = if tree { " (+ descendants)" } else { "" };
        if self.in_project_list() {
            if let Some(g) = self.current_project_group() {
                let s = summarize_group(g);
                return format!(
                    "Kill preview → project {}{} ({} processes)",
                    g.name, tree_hint, s.process_count
                );
            }
        }
        if !self.selected.is_empty() {
            return format!(
                "Kill preview: {} selected process(es){}",
                self.selected.len(),
                tree_hint
            );
        }
        let p = self.current_process();
        match p {
            Some(proc) => {
                let ports = if proc.ports.is_empty() {
                    String::new()
                } else {
                    let list = proc
                        .ports
                        .iter()
                        .map(|port| format!(":{port}"))
                        .collect::<Vec<_>>()
                        .join(",");
                    format!(" {list}")
                };
                let cmd = proc
                    .command
                    .as_deref()
                    .map(Self::format_cmd_preview)
                    .unwrap_or_default();
                format!(
                    "Kill preview → {}{} pid{}{}{}",
                    proc.name, tree_hint, proc.pid, ports, cmd
                )
            }
            None => "Nothing to kill".into(),
        }
    }

    fn format_cmd_preview(cmd: &str) -> String {
        const MAX: usize = 40;
        if cmd.len() <= MAX {
            format!(" {cmd}")
        } else {
            format!(" {}…", cmd.chars().take(MAX).collect::<String>())
        }
    }

    pub fn toggle_select_current(&mut self) {
        if let Some(pid) = self.current_pid() {
            if !self.selected.remove(&pid) {
                self.selected.insert(pid);
            }
        }
    }

    pub fn toggle_ports_only(&mut self) {
        self.ports_only = !self.ports_only;
        self.refilter();
        self.status = if self.ports_only {
            "Filter: listening ports only".into()
        } else {
            "Filter: all processes".into()
        };
    }

    pub fn pids_to_kill(&self) -> Vec<u32> {
        if self.in_project_list() {
            if let Some(g) = self.current_project_group() {
                return g.processes.iter().map(|p| p.pid).collect();
            }
            return Vec::new();
        }
        if !self.selected.is_empty() {
            self.selected.iter().copied().collect()
        } else if let Some(pid) = self.current_pid() {
            vec![pid]
        } else {
            Vec::new()
        }
    }

    pub fn apply_ports(&mut self, port_map: &[(u16, u32)]) {
        self.last_ports = port_map.to_vec();
        // Clear previous ports so stale mappings disappear after refresh.
        for p in &mut self.processes {
            p.ports.clear();
        }
        crate::process::ports::merge_ports(&mut self.processes, port_map);
        crate::process::list::sort_processes_for_display(&mut self.processes);
        if self.view_mode == ViewMode::Clean {
            self.refresh_clean_proposals();
        } else {
            self.refilter();
        }
        let with_ports = self
            .processes
            .iter()
            .filter(|p| !p.ports.is_empty())
            .count();
        self.status = format!(
            "Loaded {} listening ports → {} processes",
            port_map.len(),
            with_ports
        );
    }

    pub fn refresh(&mut self) {
        crate::process::list::refresh_process_list(&mut self.processes);
        self.selected
            .retain(|pid| self.processes.iter().any(|p| p.pid == *pid));
        if !self.last_ports.is_empty() {
            crate::process::ports::merge_ports(&mut self.processes, &self.last_ports);
        }
        crate::process::list::sort_processes_for_display(&mut self.processes);
        self.project_groups = group_projects(&self.processes);
        if self.view_mode == ViewMode::Clean {
            self.refresh_clean_proposals();
        } else {
            self.refilter();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proc(pid: u32, name: &str, ports: Vec<u16>) -> ProcessInfo {
        ProcessInfo {
            pid,
            ppid: 1,
            name: name.into(),
            cpu: 0.0,
            memory_bytes: 0,
            ports,
            command: None,
            cwd: None,
            run_time_secs: 0,
            is_zombie: false,
        }
    }

    #[test]
    fn search_matches_port_number() {
        let mut app = App::new(vec![proc(1, "node", vec![3000]), proc(2, "bash", vec![])]);
        app.query = "3000".into();
        app.refilter();
        assert_eq!(app.filtered.len(), 1);
        assert_eq!(app.processes[app.filtered[0]].pid, 1);
    }

    #[test]
    fn search_matches_colon_port() {
        let mut app = App::new(vec![proc(1, "node", vec![5173])]);
        app.query = ":5173".into();
        app.refilter();
        assert_eq!(app.filtered.len(), 1);
    }

    #[test]
    fn ports_only_filter() {
        let mut app = App::new(vec![proc(1, "node", vec![3000]), proc(2, "bash", vec![])]);
        app.toggle_ports_only();
        assert!(app.ports_only);
        assert_eq!(app.filtered.len(), 1);
        assert_eq!(app.processes[app.filtered[0]].name, "node");
    }

    #[test]
    fn table_state_follows_cursor() {
        let mut app = App::new(vec![
            proc(1, "a", vec![]),
            proc(2, "b", vec![]),
            proc(3, "c", vec![]),
        ]);
        assert_eq!(app.table_state.selected(), Some(0));
        app.move_down();
        assert_eq!(app.cursor, 1);
        assert_eq!(app.table_state.selected(), Some(1));
        app.move_down();
        app.move_down();
        assert_eq!(app.cursor, 2);
        assert_eq!(app.table_state.selected(), Some(2));
        app.move_up();
        assert_eq!(app.table_state.selected(), Some(1));
    }

    #[test]
    fn refilter_clears_table_selection_when_empty() {
        let mut app = App::new(vec![proc(1, "node", vec![3000])]);
        app.query = "nomatch".into();
        app.refilter();
        assert!(app.filtered.is_empty());
        assert_eq!(app.table_state.selected(), None);
    }

    #[test]
    fn move_first_and_last() {
        let mut app = App::new(vec![
            proc(1, "a", vec![]),
            proc(2, "b", vec![]),
            proc(3, "c", vec![]),
        ]);
        app.move_last();
        assert_eq!(app.cursor, 2);
        app.move_first();
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn move_page_down_caps_at_end() {
        let mut app = App::new(vec![
            proc(1, "a", vec![]),
            proc(2, "b", vec![]),
            proc(3, "c", vec![]),
            proc(4, "d", vec![]),
        ]);
        app.set_viewport_rows(2);
        app.move_page_down(2);
        assert_eq!(app.cursor, 2);
        app.move_page_down(2);
        assert_eq!(app.cursor, 3);
    }

    #[test]
    fn kill_preview_single_process() {
        let mut app = App::new(vec![ProcessInfo {
            pid: 4812,
            ppid: 1,
            name: "node".into(),
            cpu: 0.0,
            memory_bytes: 0,
            ports: vec![3000],
            command: Some("./node_modules/.bin/vite".into()),
            cwd: None,
            run_time_secs: 0,
            is_zombie: false,
        }]);
        app.refilter();
        let preview = app.format_kill_preview(false);
        assert!(preview.contains("node"));
        assert!(preview.contains("4812"));
        assert!(preview.contains(":3000"));
    }

    #[test]
    fn kill_preview_multi_select() {
        let mut app = App::new(vec![proc(1, "a", vec![]), proc(2, "b", vec![])]);
        app.selected.insert(1);
        app.selected.insert(2);
        let preview = app.format_kill_preview(true);
        assert!(preview.contains("2 selected"));
        assert!(preview.contains("descendants"));
    }

    #[test]
    fn request_kill_confirm_sets_pending() {
        let mut app = App::new(vec![proc(1, "node", vec![3000])]);
        app.request_kill_confirm(false, true);
        assert!(app.is_confirming_kill());
        assert_eq!(
            app.confirming_kill,
            Some(PendingKill {
                force: false,
                tree: true
            })
        );
        assert!(app.status.contains("Confirm kill?"));
    }

    #[test]
    fn request_kill_confirm_empty_clears_pending() {
        let mut app = App::new(vec![]);
        app.request_kill_confirm(false, false);
        assert!(!app.is_confirming_kill());
        assert_eq!(app.status, "Nothing to kill");
    }

    #[test]
    fn cancel_kill_confirm_clears_pending() {
        let mut app = App::new(vec![proc(1, "a", vec![])]);
        app.request_kill_confirm(true, false);
        app.cancel_kill_confirm();
        assert!(!app.is_confirming_kill());
        assert_eq!(app.status, "Kill cancelled");
    }

    #[test]
    fn take_pending_kill_consumes_state() {
        let mut app = App::new(vec![proc(1, "a", vec![])]);
        app.request_kill_confirm(true, true);
        let pending = app.take_pending_kill();
        assert_eq!(
            pending,
            Some(PendingKill {
                force: true,
                tree: true
            })
        );
        assert!(!app.is_confirming_kill());
    }

    #[test]
    fn detail_panel_lines() {
        let mut app = App::new(vec![ProcessInfo {
            pid: 4812,
            ppid: 4701,
            name: "node".into(),
            cpu: 12.4,
            memory_bytes: 421 * 1024 * 1024,
            ports: vec![3000],
            command: Some("node ./next dev".into()),
            cwd: Some("/Users/dev/my-app".into()),
            run_time_secs: 8040,
            is_zombie: false,
        }]);
        app.refilter();
        let lines = app.format_process_detail();
        assert!(lines[0].contains("4812"));
        assert!(lines[0].contains(":3000"));
        assert!(lines.iter().any(|l| l.contains("Command:")));
        assert!(lines.iter().any(|l| l.contains("Project:")));
    }

    #[test]
    fn parent_chain_builds_path() {
        let procs = vec![
            proc(1, "zsh", vec![]),
            ProcessInfo {
                pid: 2,
                ppid: 1,
                name: "bun".into(),
                cpu: 0.0,
                memory_bytes: 0,
                ports: vec![],
                command: None,
                cwd: None,
                run_time_secs: 0,
                is_zombie: false,
            },
            ProcessInfo {
                pid: 3,
                ppid: 2,
                name: "node".into(),
                cpu: 0.0,
                memory_bytes: 0,
                ports: vec![3000],
                command: None,
                cwd: None,
                run_time_secs: 0,
                is_zombie: false,
            },
        ];
        let mut app = App::new(procs);
        app.move_down();
        app.move_down();
        let lines = app.format_process_detail();
        assert!(lines.iter().any(|l| l.contains("zsh → bun → node")));
    }

    #[test]
    fn toggle_detail_requires_selection() {
        let mut app = App::new(vec![]);
        app.toggle_detail();
        assert!(!app.show_detail);
    }
    #[test]
    fn project_view_toggle() {
        let mut app = App::new(vec![ProcessInfo {
            pid: 1,
            ppid: 1,
            name: "node".into(),
            cpu: 0.0,
            memory_bytes: 0,
            ports: vec![],
            command: None,
            cwd: Some("/Users/me/my-app".into()),
            run_time_secs: 0,
            is_zombie: false,
        }]);
        app.toggle_project_view();
        assert_eq!(app.view_mode, ViewMode::Projects);
        assert!(!app.project_groups.is_empty());
    }

    #[test]
    fn project_expand_lists_members() {
        let mut app = App::new(vec![
            ProcessInfo {
                pid: 1,
                ppid: 1,
                name: "node".into(),
                cpu: 0.0,
                memory_bytes: 0,
                ports: vec![3000],
                command: None,
                cwd: Some("/Users/me/my-app".into()),
                run_time_secs: 0,
                is_zombie: false,
            },
            ProcessInfo {
                pid: 2,
                ppid: 1,
                name: "vite".into(),
                cpu: 0.0,
                memory_bytes: 0,
                ports: vec![],
                command: None,
                cwd: Some("/Users/me/my-app".into()),
                run_time_secs: 0,
                is_zombie: false,
            },
        ]);
        app.toggle_project_view();
        app.toggle_project_expand();
        assert_eq!(app.filtered.len(), 2);
    }

    #[test]
    fn project_kill_preview_whole_group() {
        let mut app = App::new(vec![ProcessInfo {
            pid: 1,
            ppid: 1,
            name: "node".into(),
            cpu: 0.0,
            memory_bytes: 0,
            ports: vec![],
            command: None,
            cwd: Some("/Users/me/my-app".into()),
            run_time_secs: 0,
            is_zombie: false,
        }]);
        app.toggle_project_view();
        let preview = app.format_kill_preview(false);
        assert!(preview.contains("project"));
        assert!(preview.contains("my-app"));
    }

    #[test]
    fn tree_view_reorders_rows_by_ppid() {
        let procs = vec![
            ProcessInfo {
                pid: 1,
                ppid: 0,
                name: "init".into(),
                cpu: 0.0,
                memory_bytes: 0,
                ports: vec![],
                command: None,
                cwd: None,
                run_time_secs: 0,
                is_zombie: false,
            },
            ProcessInfo {
                pid: 10,
                ppid: 1,
                name: "node".into(),
                cpu: 0.0,
                memory_bytes: 0,
                ports: vec![3000],
                command: None,
                cwd: None,
                run_time_secs: 0,
                is_zombie: false,
            },
            ProcessInfo {
                pid: 11,
                ppid: 10,
                name: "vite".into(),
                cpu: 0.0,
                memory_bytes: 0,
                ports: vec![],
                command: None,
                cwd: None,
                run_time_secs: 0,
                is_zombie: false,
            },
        ];
        let mut app = App::new(procs);
        app.toggle_tree_view();
        assert!(app.tree_view);
        assert_eq!(app.tree_rows.len(), 3);
        app.move_down();
        assert_eq!(app.current_process().map(|p| p.name.as_str()), Some("node"));
    }

    #[test]
    fn resources_view_with_snapshot() {
        let mut app = App::new(vec![]);
        app.resource_snapshot = ResourceSnapshot {
            available: true,
            orbstack_vm_bytes: Some(18_400_000_000),
            memory_reclaimable: Some(12_800_000_000),
            container_total_bytes: 2_500_000_000,
            ..Default::default()
        };
        app.toggle_resources_view();
        assert!(app.resources_open);
        let lines = app.resource_lines();
        assert!(lines.iter().any(|l| l.contains("VM Memory")));
        app.set_resource_panel(ResourcePanel::Containers);
        assert!(app.resource_lines().iter().any(|l| l == "Containers"));
    }
}
