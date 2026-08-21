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
            status: String::new(),
        };
        app.refilter();
        app
    }

    pub fn refilter(&mut self) {
        let q = self.query.to_lowercase();
        self.filtered = self
            .processes
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                q.is_empty()
                    || p.name.to_lowercase().contains(&q)
                    || p.ports.iter().any(|port| port.to_string().contains(&q))
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
        crate::process::ports::merge_ports(&mut self.processes, port_map);
        self.refilter();
    }

    pub fn refresh(&mut self) {
        self.processes = crate::process::list::list_processes();
        self.selected.clear();
        self.refilter();
    }
}
