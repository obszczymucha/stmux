use crate::tmux::{SplitWindowOptions, Tmux};

pub(crate) struct WorkflowImpl<'t, T: Tmux> {
    tmux: &'t T,
}

struct Candidate {
    index: String,
    name: String,
}

impl<'t, T: Tmux> WorkflowImpl<'t, T> {
    pub(crate) fn new(tmux: &'t T) -> Self {
        Self { tmux }
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

    /// Swaps `pane` with the first (forward) or last (backward) parked pane on `side`, then
    /// moves the window now holding the old pane to the other end of the queue, so repeated
    /// swaps rotate through every parked pane. Returns the name of the pane swapped in.
    fn rotate_parked(&self, pane: &str, side: &str, forward: bool) -> Option<String> {
        let current = self
            .tmux
            .get_pane_option(pane, "@window-name")
            .unwrap_or_default();

        let candidates = self.find_candidates(side);
        let (candidate, target) = if forward {
            (candidates.first()?, candidates.last()?)
        } else {
            (candidates.last()?, candidates.first()?)
        };

        self.tmux.raw(vec![
            "swap-pane",
            "-s",
            pane,
            "-t",
            format!("{}.1", candidate.index).as_str(),
        ]);
        self.tmux.raw(vec![
            "rename-window",
            "-t",
            candidate.index.as_str(),
            current.as_str(),
        ]);
        self.tmux.raw(vec![
            "move-window",
            format!("-s:{}", candidate.index).as_str(),
            format!("-t:{}", target.index).as_str(),
            if forward { "-ad" } else { "-bd" },
            ";",
            "move-window",
            "-r",
        ]);

        Some(candidate.name.clone())
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

        // The window is named after its left pane.
        if let Some(name) = self.rotate_parked(":.1", "left", forward) {
            self.tmux.raw(vec!["rename-window", name.as_str()]);
        }
    }

    fn swap_rightmost_pane(&self, forward: bool) {
        if self.tmux.count_panes() == 1 {
            return;
        }

        self.rotate_parked("{right}", "right", forward);
    }

    /// Runs right before the left pane's shell exits (`alias x='stmux workflow x; exit'`):
    /// moves the exiting pane out and the previous parked left pane in, so only the
    /// parked window closes. Also works when the left pane is alone in its window.
    fn close_left(&self) {
        if self.tmux.get_str("#{pane_left}") != "0" {
            return;
        }

        if let Some(name) = self.rotate_parked(":.1", "left", false) {
            self.tmux.raw(vec!["rename-window", name.as_str()]);
        }
    }

    fn tag_pane(&self, pane: &str, side: &str, name: &str) {
        self.tmux.raw(vec!["set-option", "-pt", pane, "@side", side]);
        self.tmux
            .raw(vec!["set-option", "-pt", pane, "@window-name", name]);
    }

    fn left_pane_name(&self) -> String {
        self.tmux
            .get_pane_option("1", "@window-name")
            .unwrap_or_else(|| self.tmux.get_str("#W"))
    }

    fn new_left(&self) {
        let old_name = self.left_pane_name();

        if self.tmux.count_panes() == 1 {
            let name = self.find_next_numeric_name();
            self.tmux.raw(vec!["new-window", "-b", "-n", name.as_str()]);
            self.tag_pane(":.1", "left", name.as_str());
            // The old window is now a parked left pane.
            self.tag_pane(":+1.1", "left", old_name.as_str());
            return;
        }

        self.tmux
            .raw(vec!["new-window", "-d", "-a", "-n", old_name.as_str()]);
        self.tmux.raw(vec!["swap-pane", "-s", ":.1", "-t", ":+1.1"]);
        self.tag_pane(":+1.1", "left", old_name.as_str());

        // Numbered after parking, so it can't collide with the parked window's name.
        let name = self.find_next_numeric_name();
        self.tmux.raw(vec!["rename-window", name.as_str()]);
        self.tag_pane(":.1", "left", name.as_str());
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

        let parked_right = self.find_candidates("right");

        if pane_count == 1
            && let Some(parked) = parked_right.first()
        {
            self.tmux
                .join_pane_to_current_window(parked.index.as_str(), 1, None, false);
        } else if pane_count == 1 {
            self.new_right();
        } else if side.is_none() && !parked_right.is_empty() {
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
            "x" => self.close_left(),
            "]" => self.shrink_right_split_or_break(),
            "[" => self.expand_right_split_or_join(),
            _ => eprintln!("No workflow defined for key: {}", key),
        }
    }
}

/// Runs the workflow against a real, throwaway tmux server (needs `tmux` installed).
#[cfg(test)]
mod tmux_tests {
    use super::*;
    use crate::command_builder::{CommandBuilder, TestCommandBuilderImpl};
    use crate::tmux::TmuxImpl;

    /// A tmux server on its own socket, killed on drop (also when a test panics).
    /// Starts with session `t`, window 1 named "1" whose single pane is tagged left/1.
    struct Server {
        cb: TestCommandBuilderImpl,
    }

    impl Server {
        fn new(test: &str) -> Self {
            let socket = format!("stmux-test-{}-{}", std::process::id(), test);
            let server = Self {
                cb: TestCommandBuilderImpl::new(&socket),
            };
            server.run(&[
                "start-server", ";",
                "set", "-g", "base-index", "1", ";",
                "set", "-g", "pane-base-index", "1", ";",
                "set", "-g", "@window-names-right", "a b c", ";",
                "new-session", "-d", "-s", "t", "-x", "200", "-y", "50", "-n", "1",
            ]);
            server.tag(":1.1", "left", "1");
            server
        }

        fn run(&self, args: &[&str]) -> String {
            let out = self.cb.new_command().args(args).output().unwrap();
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        }

        fn key(&self, key: &str) {
            let tmux = TmuxImpl::new(&self.cb);
            WorkflowImpl::new(&tmux).on_action(key);
        }

        fn tag(&self, pane: &str, side: &str, name: &str) {
            self.run(&["set", "-p", "-t", pane, "@side", side]);
            self.run(&["set", "-p", "-t", pane, "@window-name", name]);
        }

        /// Appends a background single-pane window, the way the workflow parks panes.
        fn park(&self, name: &str, side: &str) {
            let index = self.run(&["new-window", "-d", "-P", "-F", "#I", "-n", name]);
            self.tag(&format!(":{}.1", index), side, name);
        }

        /// Splits the current window; the new right pane becomes active.
        fn split(&self, tag: Option<(&str, &str)>) {
            self.run(&["split-window", "-h"]);
            if let Some((side, name)) = tag {
                self.tag(":.2", side, name);
            }
        }

        fn focus_left(&self) {
            self.run(&["select-pane", "-t", ":.1"]);
        }

        /// One line per pane: `index:window-name[*=current] pane-id side/window-name`.
        fn state(&self) -> Vec<String> {
            self.run(&[
                "list-panes",
                "-s",
                "-F",
                "#I:#W#{?window_active,*,} #{pane_id} #{@side}/#{@window-name}",
            ])
            .lines()
            .map(String::from)
            .collect()
        }
    }

    impl Drop for Server {
        fn drop(&mut self) {
            // kill-server leaves the socket file behind.
            let socket = self.run(&["display-message", "-p", "#{socket_path}"]);
            self.cb.kill_server();
            let _ = std::fs::remove_file(socket);
        }
    }

    // --- 1 / 2: swap_active_pane ---

    #[test]
    fn swap_does_nothing_with_a_single_pane() {
        let s = Server::new("swap-single");
        s.park("2", "left");
        let before = s.state();

        s.key("1");
        s.key("2");

        assert_eq!(s.state(), before);
    }

    #[test]
    fn swap_left_brings_in_the_first_parked_left_pane_and_queues_the_old_one() {
        let s = Server::new("swap-left");
        s.split(Some(("right", "a")));
        s.park("2", "left");
        s.park("3", "left");
        s.focus_left();

        s.key("2");

        assert_eq!(
            s.state(),
            [
                "1:2* %2 left/2",
                "1:2* %1 right/a",
                "2:3 %3 left/3",
                "3:1 %0 left/1",
            ]
        );
    }

    #[test]
    fn swap_left_forward_cycles_through_all_parked_left_panes() {
        let s = Server::new("swap-left-cycle");
        s.split(Some(("right", "a")));
        s.park("2", "left");
        s.park("3", "left");
        s.focus_left();

        let mut seen = vec![];
        for _ in 0..3 {
            s.key("2");
            seen.push(s.run(&["show-option", "-t", ":.1", "-pv", "@window-name"]));
        }

        assert_eq!(seen, ["2", "3", "1"]);
    }

    #[test]
    fn swap_left_backward_cycles_the_other_way() {
        let s = Server::new("swap-left-cycle-back");
        s.split(Some(("right", "a")));
        s.park("2", "left");
        s.park("3", "left");
        s.focus_left();

        let mut seen = vec![];
        for _ in 0..3 {
            s.key("1");
            seen.push(s.run(&["display-message", "-p", "#W"]));
        }

        assert_eq!(seen, ["3", "2", "1"]);
    }

    #[test]
    fn swap_left_after_new_left_swaps_back_to_the_old_window() {
        let s = Server::new("swap-left-after-q");
        s.run(&["set", "-pu", "-t", ":1.1", "@side"]);
        s.run(&["set", "-pu", "-t", ":1.1", "@window-name"]);

        s.key("q"); // single pane: opens window 2 and parks window 1
        s.split(Some(("right", "a")));
        s.focus_left();
        s.key("2");

        assert_eq!(
            s.state(),
            ["1:1* %0 left/1", "1:1* %2 right/a", "2:2 %1 left/2"]
        );
    }

    #[test]
    fn swap_left_ignores_parked_right_panes() {
        let s = Server::new("swap-left-ignores-right");
        s.split(Some(("right", "a")));
        s.park("b", "right");
        s.focus_left();
        let before = s.state();

        s.key("2");

        assert_eq!(s.state(), before);
    }

    #[test]
    fn swap_right_forward_rotates_parked_right_panes() {
        let s = Server::new("swap-right-fwd");
        s.split(Some(("right", "a")));
        s.park("b", "right");
        s.park("c", "right");

        s.key("2");

        assert_eq!(
            s.state(),
            [
                "1:1* %0 left/1",
                "1:1* %2 right/b",
                "2:c %3 right/c",
                "3:a %1 right/a",
            ]
        );
    }

    #[test]
    fn swap_right_backward_rotates_the_other_way() {
        let s = Server::new("swap-right-back");
        s.split(Some(("right", "a")));
        s.park("b", "right");
        s.park("c", "right");

        s.key("1");

        assert_eq!(
            s.state(),
            [
                "1:1* %0 left/1",
                "1:1* %3 right/c",
                "2:a %1 right/a",
                "3:b %2 right/b",
            ]
        );
    }

    #[test]
    fn swap_right_with_one_parked_pane_swaps_it_in() {
        let s = Server::new("swap-right-one");
        s.split(Some(("right", "a")));
        s.park("b", "right");

        s.key("2");

        assert_eq!(
            s.state(),
            ["1:1* %0 left/1", "1:1* %2 right/b", "2:a %1 right/a"]
        );
    }

    #[test]
    fn swap_right_does_nothing_without_parked_right_panes() {
        let s = Server::new("swap-right-none");
        s.split(Some(("right", "a")));
        s.park("2", "left");
        let before = s.state();

        s.key("2");

        assert_eq!(s.state(), before);
    }

    // --- 3: new_right ---

    #[test]
    fn new_right_splits_a_single_pane_with_the_first_free_name() {
        let s = Server::new("new-right-split");

        s.key("3");

        assert_eq!(s.state(), ["1:1* %0 left/1", "1:1* %1 right/a"]);
    }

    #[test]
    fn new_right_skips_names_already_used_by_windows() {
        let s = Server::new("new-right-skip");
        s.park("a", "right");

        s.key("3");

        assert_eq!(
            s.state(),
            ["1:1* %0 left/1", "1:1* %2 right/b", "2:a %1 right/a"]
        );
    }

    #[test]
    fn new_right_parks_the_current_right_pane() {
        let s = Server::new("new-right-park");
        s.split(Some(("right", "a")));

        s.key("3");

        assert_eq!(
            s.state(),
            ["1:1* %0 left/1", "1:1* %2 right/b", "2:a %1 right/a"]
        );
    }

    #[test]
    fn new_right_does_nothing_when_all_names_are_taken() {
        let s = Server::new("new-right-full");
        s.split(Some(("right", "a")));
        s.park("b", "right");
        s.park("c", "right");
        let before = s.state();

        s.key("3");

        assert_eq!(s.state(), before);
    }

    #[test]
    fn new_right_with_single_pane_does_nothing_when_all_names_are_taken() {
        let s = Server::new("new-right-full-single");
        s.park("a", "right");
        s.park("b", "right");
        s.park("c", "right");
        let before = s.state();

        s.key("3");

        assert_eq!(s.state(), before);
    }

    // --- q: new_left ---

    #[test]
    fn new_left_with_single_pane_opens_a_numbered_window_before_the_current_one() {
        let s = Server::new("new-left-single");

        s.key("q");

        assert_eq!(s.state(), ["1:2* %1 left/2", "2:1 %0 left/1"]);
    }

    #[test]
    fn new_left_numbers_after_the_highest_numeric_window() {
        let s = Server::new("new-left-numbering");
        s.park("7", "left");
        s.park("b", "right");

        s.key("q");

        assert_eq!(s.state()[0], "1:8* %3 left/8");
    }

    #[test]
    fn new_left_with_two_panes_parks_the_left_pane() {
        let s = Server::new("new-left-park");
        s.split(Some(("right", "a")));

        s.key("q");

        assert_eq!(
            s.state(),
            [
                "1:2* %2 left/2",
                "1:2* %1 right/a",
                "2:1 %0 left/1",
            ]
        );
    }

    #[test]
    fn new_left_with_untagged_left_pane_does_not_duplicate_names() {
        let s = Server::new("new-left-untagged");
        s.run(&["set", "-pu", "-t", ":1.1", "@window-name"]);
        s.split(Some(("right", "a")));

        s.key("q");

        assert_eq!(
            s.state(),
            ["1:2* %2 left/2", "1:2* %1 right/a", "2:1 %0 left/1"]
        );
    }

    #[test]
    fn new_left_skips_numbers_taken_by_other_windows() {
        let s = Server::new("new-left-taken");
        s.park("2", "left");
        s.split(Some(("right", "a")));

        s.key("q");

        assert_eq!(s.run(&["display-message", "-p", "#W"]), "3");
    }

    // --- w: toggle ---

    #[test]
    fn toggle_splits_when_nothing_is_parked() {
        let s = Server::new("toggle-split");

        s.key("w");

        assert_eq!(s.state(), ["1:1* %0 left/1", "1:1* %1 right/a"]);
    }

    #[test]
    fn toggle_breaks_a_tagged_right_pane_out() {
        let s = Server::new("toggle-break");
        s.split(Some(("right", "a")));

        s.key("w");

        assert_eq!(s.state(), ["1:1* %0 left/1", "2:a %1 right/a"]);
    }

    #[test]
    fn toggle_joins_the_next_parked_right_pane() {
        let s = Server::new("toggle-join");
        s.park("a", "right");

        s.key("w");

        assert_eq!(s.state(), ["1:1* %0 left/1", "1:1* %1 right/a"]);
    }

    #[test]
    fn toggle_join_keeps_the_left_pane() {
        let s = Server::new("toggle-join-keeps-left");
        s.park("2", "left");
        s.park("a", "right");

        s.key("w");

        assert_eq!(
            s.state(),
            ["1:1* %0 left/1", "1:1* %2 right/a", "2:2 %1 left/2"]
        );
    }

    #[test]
    fn toggle_joins_a_right_pane_parked_before_the_current_window() {
        let s = Server::new("toggle-join-before");
        s.park("a", "right");
        s.run(&["swap-window", "-s", ":1", "-t", ":2"]);
        s.run(&["select-window", "-t", ":2"]);

        s.key("w");

        assert_eq!(s.state(), ["2:1* %0 left/1", "2:1* %1 right/a"]);
    }

    #[test]
    fn toggle_round_trip_restores_the_right_pane() {
        let s = Server::new("toggle-round-trip");

        s.key("w");
        s.key("w");
        s.key("w");

        assert_eq!(s.state(), ["1:1* %0 left/1", "1:1* %1 right/a"]);
    }

    #[test]
    fn toggle_swaps_an_untagged_right_pane_for_a_parked_one() {
        let s = Server::new("toggle-untagged-swap");
        s.split(None);
        s.park("b", "right");

        s.key("w");

        assert_eq!(s.state(), ["1:1* %0 left/1", "1:1* %2 right/b", "2: %1 /"]);
    }

    #[test]
    fn toggle_replaces_an_untagged_right_pane_when_nothing_is_parked() {
        let s = Server::new("toggle-untagged-new");
        s.split(None);

        s.key("w");

        assert_eq!(s.state(), ["1:1* %0 left/1", "1:1* %2 right/a", "2:a %1 right/"]);
    }

    // --- x: close_left ---

    #[test]
    fn close_left_swaps_in_the_previous_parked_left_pane_before_exit() {
        let s = Server::new("close-left");
        s.park("2", "left");
        s.park("3", "left");

        s.key("x");
        assert_eq!(
            s.state(),
            ["1:3* %2 left/3", "2:1 %0 left/1", "3:2 %1 left/2"]
        );

        s.run(&["kill-pane", "-t", "%0"]); // what `exit` does to the moved shell
        assert_eq!(s.state(), ["1:3* %2 left/3", "3:2 %1 left/2"]);
    }

    #[test]
    fn close_left_keeps_the_right_pane() {
        let s = Server::new("close-left-split");
        s.split(Some(("right", "a")));
        s.park("2", "left");
        s.focus_left();

        s.key("x");

        assert_eq!(
            s.state(),
            ["1:2* %2 left/2", "1:2* %1 right/a", "2:1 %0 left/1"]
        );
    }

    #[test]
    fn close_left_does_nothing_from_the_right_pane() {
        let s = Server::new("close-left-from-right");
        s.split(Some(("right", "a")));
        s.park("2", "left");
        s.park("b", "right");
        let before = s.state();

        s.key("x");

        assert_eq!(s.state(), before);
    }

    #[test]
    fn close_left_does_nothing_without_parked_left_panes() {
        let s = Server::new("close-left-none");
        s.park("b", "right");
        let before = s.state();

        s.key("x");

        assert_eq!(s.state(), before);
    }

    // --- unimplemented / unknown keys ---

    #[test]
    fn unimplemented_and_unknown_keys_change_nothing() {
        let s = Server::new("noop-keys");
        s.split(Some(("right", "a")));
        let before = s.state();

        for key in ["]", "[", "z"] {
            s.key(key);
        }

        assert_eq!(s.state(), before);
    }
}
