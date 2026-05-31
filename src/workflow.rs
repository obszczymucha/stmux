use crate::tmux::{SplitWindowOptions, Tmux};

pub(crate) struct WorkflowImpl<'t, T: Tmux> {
    tmux: &'t T,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
struct Candidate {
    index: String,
    name: String,
}

struct NameAndSide {
    name: String,
    side: String,
}

impl<'t, T: Tmux> WorkflowImpl<'t, T> {
    pub(crate) fn new(tmux: &'t T) -> Self {
        Self { tmux }
    }

    fn list_numeric_windows(&self) -> Vec<usize> {
        self.tmux
            .list_windows_for_current_session("#W")
            .into_iter()
            .filter_map(|window| window.parse::<usize>().ok())
            .collect()
    }

    fn parse_names(&self, names_str: &str) -> Vec<String> {
        names_str.split(' ').map(|s| s.trim().to_string()).collect()
    }

    fn get_windows_with_sides(&self) -> Vec<NameAndSide> {
        self.tmux
            .raw_vec(vec!["list-windows", "-F", "#W:#{@side}"])
            .into_iter()
            .filter_map(|entry| {
                let mut parts = entry.splitn(2, ':');
                let name = parts.next().unwrap_or("");
                let side = parts.next().unwrap_or("");
                if !name.is_empty() {
                    Some(NameAndSide {
                        name: name.to_string(),
                        side: side.to_string(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    fn find_next_right_window_name(&self) -> Option<String> {
        let current_window_name = self.tmux.get_str("#W");
        let windows = self.get_windows_with_sides();
        let mut current = false;

        for entry in windows {
            // eprintln!("Window: {}, Side: {}", entry.name, entry.side);
            if entry.name == current_window_name {
                current = true;
            } else if current && entry.side == "right" {
                return Some(entry.name);
            }
        }

        None
    }

    fn find_next_right_name(&self) -> Option<String> {
        let mut windows = self.tmux.raw_vec(vec!["list-windows", "-F", "#W"]);
        let right_pane_name =
            self.tmux
                .raw_str_opt(vec!["display-message", "-p", "-t:.2", "#{@window-name}"]);
        if let Some(right_name) = right_pane_name {
            windows.push(right_name);
        }

        let names_str = self.tmux.get_str("#{@window-names-right}");
        let names = self.parse_names(names_str.as_str());

        if names.is_empty() {
            return None;
        }

        for name in &names {
            if !windows.contains(name) {
                return Some(name.clone());
            }
        }

        None
    }

    fn find_next_numeric_name(&self) -> String {
        let windows = &mut self.list_numeric_windows();
        windows.sort();
        let max = windows.last();

        let result = if let Some(value) = max { value + 1 } else { 1 };
        format!("{}", result)
    }

    fn find_next_alpha_name(&self) -> String {
        let windows = self.tmux.raw_vec(vec!["list-windows", "-F", "#W"]);

        for c in b'a'..=b'z' {
            let name = (c as char).to_string();

            if !windows.contains(&name) {
                return name;
            }
        }

        String::from("a")
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
                let pane_count = parts.next().unwrap_or("0");
                let count = pane_count.parse::<usize>().unwrap_or(0);
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

    fn list_alpha_windows(&self) -> Vec<String> {
        self.tmux
            .list_windows_for_current_session("#W")
            .into_iter()
            .filter(|window| !window.chars().all(char::is_numeric))
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
        let count = self.tmux.count_panes();
        if count == 1 {
            return;
        }

        let pane_left = self.tmux.get_str("#{pane_left}");
        let is_leftmost = pane_left == "0";

        if is_leftmost {
            eprintln!("chuj");
            let current_name = self
                .tmux
                .get_pane_option("1", "@window-name")
                .unwrap_or_default();
            let mut candidates = self.find_candidates("left");
            candidates.sort_by_key(|s| s.index.parse::<usize>().unwrap_or(0));
            candidates
                .iter()
                .for_each(|c| eprintln!("Candidate: {}", c.name));
            if let Some(target) = Self::cycle_candidate(&current_name, &candidates, forward) {
                eprintln!("Swapping with target: {}", target.name);
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

            return;
        }

        let current = self
            .tmux
            .get_pane_option("2", "@window-name")
            .unwrap_or_default();

        eprintln!("window-name: {}", current);
        let candidates = self.find_candidates("right");
        candidates
            .iter()
            .for_each(|c| eprintln!("Candidate: {}", c.name));

        let candidate = if forward {
            candidates.first()
        } else {
            candidates.last()
        };

        let target = if forward {
            candidates.last()
        } else {
            candidates.first()
        };

        if let Some(c) = candidate {
            self.tmux
                .raw(vec!["swap-pane", "-t", format!("{}.1", c.index).as_str()]);
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

    // Aka: select previous window binding
    fn previous(&self) {
        self.swap_active_pane(false);
    }

    // Aka: select next window binding
    fn next(&self) {
        self.swap_active_pane(true);
    }

    fn swap_pane(&self, swap_window_name: &str) {
        let current_window_index = self.tmux.get_str("#{window_index}");
        let this_index = format!("{}.2", current_window_index);
        let this_window_name = self
            .tmux
            .get_pane_option(this_index.as_str(), "@window-name")
            .unwrap_or("chuj".to_string());
        let that_index = format!("{}.1", swap_window_name);
        let that_window_name = self
            .tmux
            .get_pane_option(that_index.as_str(), "@window-name")
            .unwrap_or("chuj".to_string());
        self.tmux
            .swap_panes(&current_window_index, 2, swap_window_name, 1);
        self.tmux.set_pane_option(
            swap_window_name,
            1,
            "@window-name",
            this_window_name.as_str(),
        );
        self.tmux.set_pane_option(
            current_window_index.as_str(),
            2,
            "@window-name",
            that_window_name.as_str(),
        );
        self.tmux
            .rename_window_in_current_session(swap_window_name, this_window_name.as_str());
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
        // self.tmux.raw(vec!["display-message", format!("Current path: {}", path).as_str()]);
        let options = SplitWindowOptions {
            horizontally: true,
            path: Some(path),
            startup_command: None,
            at_index: None,
            before: false,
        };

        let name = self.find_next_right_name();

        if let Some(name) = name {
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
        let count = self.tmux.count_panes();

        if count == 1 {
            self.new_pane();
            return;
        }

        let old_name = self.tmux.get_pane_option("2", "@window-name");
        let name_opt = self.find_next_right_name();
        if name_opt.is_none() {
            return;
        }

        let name = name_opt.unwrap();

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

    fn test(&self) {
        let name = self.find_next_right_name();

        if let Some(name) = name {
            eprintln!("Next right name: {}", name);
        } else {
            eprintln!("No next right name found");
        }
    }

    fn toggle(&self) {
        let pane_count = self.tmux.count_panes();
        // let next_window_name = self
        //     .tmux
        //     .raw_str_opt(vec!["display-message", "-p", "-t:+", "#W"]);
        if pane_count == 1
            && let Some(name) = self.find_next_right_window_name()
        {
            self.tmux
                .join_pane_to_current_window(name.as_str(), 1, None, false);
        } else if pane_count == 1 {
            self.new_right();
        } else {
            self.break_pane();
        }
    }

    pub fn on_action(&self, key: &str) {
        match key {
            "1" => self.previous(),
            "2" => self.next(),
            "3" => self.new_right(),
            "q" => self.new_left(),
            "w" => self.toggle(),
            "]" => self.shrink_right_split_or_break(),
            "[" => self.expand_right_split_or_join(),
            _ => eprintln!("No workflow defined for key: {}", key),
        }
    }
}
