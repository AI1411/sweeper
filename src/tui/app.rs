use std::collections::HashSet;

use crate::process::ProcessInfo;

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
        };
        app.refilter();
        app
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
        }
    }

    pub fn move_down(&mut self) {
        if !self.filtered.is_empty() && self.cursor + 1 < self.filtered.len() {
            self.cursor += 1;
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
}
