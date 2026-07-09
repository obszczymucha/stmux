use crate::tmux::{SplitWindowOptions, Tmux};

pub(crate) struct WorkflowImpl<'t, T: Tmux> {
    tmux: &'t T,
}

#[derive(Clone)]
struct Candidate {
    index: String,
    name: String,
}

impl<'t, T: Tmux> WorkflowImpl<'t, T> {
    pub(crate) fn new(tmux: &'t T) -> Self {
        Self { tmux }
    }

    fn find_next_right_window_name(&self) -> Option<String> {
        let current_window_name = self.tmux.get_str("#W");
        let mut seen_current = false;

        for entry in self.tmux.raw_vec(vec!["list-windows", "-F", "#W:#{@side}"]) {
            let (name, side) = entry.split_once(':').unwrap_or((entry.as_str(), ""));
            if name == current_window_name {
                seen_current = true;
            } else if seen_current && side == "right" {
                return Some(name.to_string());
            }
        }

        None
    }

    fn find_next_right_name(&self) -> Option<String> {
        let mut windows = self.tmux.raw_vec(vec!["list-windows", "-F", "#W"]);
        if let Some(right_name) =
            self.tmux
                .raw_str_opt(vec!["display-message", "-p", "-t:.2", "#{@window-name}"])
        {
            windows.push(right_name);
        }

        let names = self.tmux.get_str("#{@window-names-right}");
        names
            .split(' ')
            .map(str::trim)
            .find(|name| !name.is_empty() && !windows.iter().any(|w| w == name))
            .map(String::from)
    }

    fn find_next_numeric_name(&self) -> String {
        let max = self
            .tmux
            .list_windows_for_current_session("#W")
            .into_iter()
            .filter_map(|window| window.parse::<usize>().ok())
            .max()
            .unwrap_or(0);

        (max + 1).to_string()
    }

    fn find_candidates(&self, side: &str) -> Vec<Candidate> {
        let current_index = self.tmux.get_str("#I");

        self.tmux
            .raw_vec(vec!["list-windows", "-F", "#I:#W:#{window_panes}:#{@side}"])
            .into_iter()
            .filter_map(|entry| {
                let mut parts = entry.splitn(4, ':');
                let index = parts.next().unwrap_or("");
                let name = parts.next().unwrap_or("");
                let count = parts.next().unwrap_or("0").parse::<usize>().unwrap_or(0);
                let s = parts.next().unwrap_or("unknown");
                if index != current_index && count == 1 && s == side {
                    Some(Candidate {
                        index: index.to_string(),
                        name: name.to_string(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    fn cycle_candidate(current: &str, list: &[Candidate], forward: bool) -> Option<Candidate> {
        if list.is_empty() {
            return None;
        }
        let pos = list.iter().position(|x| x.name == current);
        let next_pos = match pos {
            Some(i) => {
                if forward {
                    (i + 1) % list.len()
                } else {
                    (i + list.len() - 1) % list.len()
                }
            }
            None => 0,
        };
        Some(list[next_pos].clone())
    }

    fn swap_active_pane(&self, forward: bool) {
        if self.tmux.count_panes() == 1 {
            return;
        }

        if self.tmux.get_str("#{pane_left}") != "0" {
            // Not the leftmost pane, so it's the right one in a two-pane layout.
            self.swap_rightmost_pane(forward);
            return;
        }

        let current_name = self
            .tmux
            .get_pane_option("1", "@window-name")
            .unwrap_or_default();
        let mut candidates = self.find_candidates("left");
        candidates.sort_by_key(|c| c.index.parse::<usize>().unwrap_or(0));

        if let Some(target) = Self::cycle_candidate(&current_name, &candidates, forward) {
            self.tmux.raw(vec![
                "swap-pane",
                "-s",
                ":.1",
                "-t",
                &format!("{}.1", target.index),
            ]);
            self.tmux
                .set_pane_option_for_current_window(1, "@window-name", &target.name);
            self.tmux.raw(vec!["rename-window", &target.name]);
            self.tmux.raw(vec![
                "rename-window",
                "-t",
                &format!("{}.1", target.index),
                &current_name,
            ]);
        }
    }

    fn swap_rightmost_pane(&self, forward: bool) {
        if self.tmux.count_panes() == 1 {
            return;
        }

        let current = self
            .tmux
            .get_pane_option("{right}", "@window-name")
            .unwrap_or_default();

        let candidates = self.find_candidates("right");
        let (candidate, target) = if forward {
            (candidates.first(), candidates.last())
        } else {
            (candidates.last(), candidates.first())
        };

        if let Some(c) = candidate {
            self.tmux.raw(vec![
                "swap-pane",
                "-s",
                "{right}",
                "-t",
                format!("{}.1", c.index).as_str(),
            ]);
            self.tmux.raw(vec![
                "rename-window",
                "-t",
                c.index.as_str(),
                current.as_str(),
            ]);

            if let Some(t) = target {
                self.tmux.raw(vec![
                    "move-window",
                    format!("-s:{}", c.index).as_str(),
                    format!("-t:{}", t.index).as_str(),
                    if forward { "-ad" } else { "-bd" },
                    ";",
                    "move-window",
                    "-r",
                ]);
            }
        }
    }

    fn new_left(&self) {
        let count = self.tmux.count_panes();

        if count == 1 {
            let name = self.find_next_numeric_name();
            self.tmux.raw(vec!["new-window", "-b", "-n", name.as_str()]);
            self.tmux.raw(vec!["set-option", "-p", "@side", "left"]);
            self.tmux
                .raw(vec!["set-option", "-p", "@window-name", name.as_str()]);
            return;
        }

        let old_name = self.tmux.get_pane_option("1", "@window-name");
        let name = self.find_next_numeric_name();
        self.tmux
            .raw(vec!["new-window", "-d", "-a", "-n", name.as_str()]);
        self.tmux.raw(vec!["swap-pane", "-s", ":.1", "-t", ":+1.1"]);
        self.tmux
            .raw(vec!["set-option", "-pt", ":+1.1", "@side", "left"]);
        if let Some(ref old) = old_name {
            self.tmux.raw(vec![
                "set-option",
                "-pt",
                ":+1.1",
                "@window-name",
                old.as_str(),
            ]);
            self.tmux.raw(vec!["rename-window", "-t:+1", old.as_str()]);
        }
        let new_pane_name = self.find_next_numeric_name();
        self.tmux
            .set_pane_option_for_current_window(1, "@window-name", new_pane_name.as_str());
        self.tmux
            .set_pane_option_for_current_window(1, "@side", "left");
    }

    fn new_pane(&self) {
        let path = self.tmux.get_str("#{pane_current_path}");
        let options = SplitWindowOptions {
            horizontally: true,
            path: Some(path),
            startup_command: None,
            at_index: None,
            before: false,
        };

        if let Some(name) = self.find_next_right_name() {
            self.tmux.split_current_window(&options);

            self.tmux
                .set_pane_option_for_current_window(2, "@window-name", name.as_str());
            self.tmux
                .set_pane_option_for_current_window(2, "@side", "right");
        }
    }

    fn break_pane(&self) {
        let name = self.tmux.get_pane_option("2", "@window-name");

        self.tmux.raw(vec!["break-pane", "-s:.2", "-ad"]);

        if let Some(name) = name {
            self.tmux.raw(vec!["rename-window", "-t:+1", name.as_str()]);
        }
    }

    fn new_right(&self) {
        if self.tmux.count_panes() == 1 {
            self.new_pane();
            return;
        }

        let old_name = self.tmux.get_pane_option("2", "@window-name");
        let Some(name) = self.find_next_right_name() else {
            return;
        };

        let path = self.tmux.get_str("#{pane_current_path}");
        self.tmux.raw(vec![
            "new-window",
            "-d",
            "-a",
            "-c",
            path.as_str(),
            "-n",
            name.as_str(),
        ]);
        self.tmux.raw(vec!["swap-pane", "-s", ":.2", "-t", ":+1.1"]);
        self.tmux
            .raw(vec!["set-option", "-pt", ":+1.1", "@side", "right"]);
        if let Some(ref old) = old_name {
            self.tmux.raw(vec![
                "set-option",
                "-pt",
                ":+1.1",
                "@window-name",
                old.as_str(),
            ]);
            self.tmux.raw(vec!["rename-window", "-t:+1", old.as_str()]);
        }
        self.tmux
            .set_pane_option_for_current_window(2, "@window-name", name.as_str());
        self.tmux
            .set_pane_option_for_current_window(2, "@side", "right");
    }

    fn shrink_right_split_or_break(&self) {
        eprintln!("Executing workflow for key: ]");
    }

    fn expand_right_split_or_join(&self) {
        eprintln!("Executing workflow for key: [");
    }

    fn toggle(&self) {
        let pane_count = self.tmux.count_panes();
        let side = self
            .tmux
            .raw_str_opt(vec!["show-option", "-t", "{right}", "-p", "@side"]);

        if pane_count == 1
            && let Some(name) = self.find_next_right_window_name()
        {
            self.tmux
                .join_pane_to_current_window(name.as_str(), 1, None, false);
            self.swap_active_pane(true);
        } else if pane_count == 1 {
            self.new_right();
        } else if side.is_none() && self.find_next_right_window_name().is_some() {
            self.swap_rightmost_pane(true);
        } else if side.is_none() {
            self.new_right();
        } else {
            self.break_pane();
        }
    }

    pub fn on_action(&self, key: &str) {
        match key {
            // 1 and 2 are the select previous/next window bindings.
            "1" => self.swap_active_pane(false),
            "2" => self.swap_active_pane(true),
            "3" => self.new_right(),
            "q" => self.new_left(),
            "w" => self.toggle(),
            "]" => self.shrink_right_split_or_break(),
            "[" => self.expand_right_split_or_join(),
            _ => eprintln!("No workflow defined for key: {}", key),
        }
    }
}
