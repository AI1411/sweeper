use std::collections::HashSet;

use ratatui::widgets::TableState;

use crate::process::ProcessInfo;

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

    pub fn sync_table_state(&mut self) {
        if self.filtered.is_empty() {
            self.table_state.select(None);
        } else {
            self.table_state.select(Some(self.cursor));
        }
    }

    pub fn refilter(&mut self) {
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
        self.sync_table_state();
    }

    pub fn current_pid(&self) -> Option<u32> {
        self.filtered
            .get(self.cursor)
            .and_then(|i| self.processes.get(*i))
            .map(|p| p.pid)
    }

    pub fn move_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.sync_table_state();
        }
    }

    pub fn move_down(&mut self) {
        if !self.filtered.is_empty() && self.cursor + 1 < self.filtered.len() {
            self.cursor += 1;
            self.sync_table_state();
        }
    }

    pub fn move_first(&mut self) {
        if !self.filtered.is_empty() {
            self.cursor = 0;
            self.sync_table_state();
        }
    }

    pub fn move_last(&mut self) {
        if !self.filtered.is_empty() {
            self.cursor = self.filtered.len() - 1;
            self.sync_table_state();
        }
    }

    pub fn move_page_up(&mut self, step: usize) {
        if self.filtered.is_empty() {
            return;
        }
        let step = step.max(1);
        self.cursor = self.cursor.saturating_sub(step);
        self.sync_table_state();
    }

    pub fn move_page_down(&mut self, step: usize) {
        if self.filtered.is_empty() {
            return;
        }
        let step = step.max(1);
        let max = self.filtered.len() - 1;
        self.cursor = (self.cursor + step).min(max);
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
        if !self.selected.is_empty() {
            return format!(
                "Kill preview: {} selected process(es){}",
                self.selected.len(),
                tree_hint
            );
        }
        let p = self
            .filtered
            .get(self.cursor)
            .and_then(|i| self.processes.get(*i));
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
        self.refilter();
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
        self.processes = crate::process::list::list_processes();
        self.selected.clear();
        if !self.last_ports.is_empty() {
            crate::process::ports::merge_ports(&mut self.processes, &self.last_ports);
        }
        self.refilter();
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
}
