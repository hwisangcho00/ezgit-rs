use git2::Repository;

pub enum Panel {
    CommitLog,
    Branches,
}

#[derive(PartialEq)]
pub enum UIState {
    Normal,
    CommitMessage,
    ConfirmCommit,
    ConfirmQuit,
    CommitDetails,
    CreateBranch,
    KeyGuide,
    ConfirmMerge,
    Error,
}

pub struct CommitState {
    pub message: String,
}

pub enum InputMode {
    Command, // Default mode for handling commands
    Text,    // Mode for handling text input
}

pub struct AppState {
    pub selected_index: usize,         // Selected commit index
    pub commit_log: Vec<String>,       // Commit log
    pub visible_range: (usize, usize), // Visible range of commits
    pub visible_count: usize,
    pub horizontal_offset: usize,
    pub branches: Vec<String>,  // Branch list
    pub selected_branch: usize, // Selected branch index
    pub branch_visible_range: (usize, usize),
    pub branch_visible_count: usize,
    pub focused_panel: Panel, // Currently focused panel
    pub selected_commit_details: Option<String>,
    pub ui_state: UIState,
    pub commit_state: Option<CommitState>,
    pub commit_details_visible_range: (usize, usize),
    pub commit_details_total_lines: usize,
    pub input_mode: InputMode,
    pub branch_name: String,
    pub error_message: Option<String>,
}

impl AppState {
    pub fn new(commit_log: Vec<String>, branches: Vec<String>, repo_path: &str) -> Self {
        // Determine the current branch
        let current_branch = match Repository::open(repo_path).and_then(|repo| {
            repo.head().and_then(|head| {
                head.shorthand()
                    .map(String::from)
                    .ok_or(git2::Error::from_str("No branch name"))
            })
        }) {
            Ok(branch_name) => branch_name,
            Err(_) => String::new(), // Default to an empty string if the branch can't be determined
        };

        // Find the index of the current branch in the branches list
        let selected_branch = branches
            .iter()
            .position(|branch| branch == &current_branch)
            .unwrap_or(0); // Default to the first branch if the current branch isn't found

        Self {
            selected_index: 0,
            commit_log,
            visible_range: (0, 0),
            visible_count: 10,
            horizontal_offset: 0,
            branches,
            selected_branch,
            branch_visible_range: (0, 0),
            branch_visible_count: 10,
            focused_panel: Panel::CommitLog,
            selected_commit_details: None,
            ui_state: UIState::Normal,
            commit_state: None,
            commit_details_visible_range: (0, 0),
            commit_details_total_lines: 0,
            input_mode: InputMode::Command,
            branch_name: current_branch,
            error_message: None,
        }
    }

    pub fn update_visible_range(&mut self) {
        let start = self
            .selected_index
            .saturating_sub(self.selected_index % self.visible_count);
        let end = usize::min(start + self.visible_count, self.commit_log.len());
        self.visible_range = (start, end);
    }

    pub fn scroll_up(&mut self) {
        if self.visible_range.0 > 0 {
            self.visible_range.0 -= 1;
            self.visible_range.1 = usize::min(self.visible_range.1 - 1, self.commit_log.len());
        }
    }

    pub fn scroll_down(&mut self) {
        if self.visible_range.1 < self.commit_log.len() {
            self.visible_range.0 += 1;
            self.visible_range.1 = usize::min(self.visible_range.1 + 1, self.commit_log.len());
        }
    }

    pub fn update_branch_visible_range(&mut self) {
        let start = self
            .selected_branch
            .saturating_sub(self.branch_visible_count / 2);
        let end = (start + self.branch_visible_count).min(self.branches.len());
        self.branch_visible_range = (start, end);
    }

    pub fn select_previous_branch(&mut self) {
        if self.selected_branch > 0 {
            self.selected_branch -= 1;
            self.update_branch_visible_range();
        }
    }

    pub fn select_next_branch(&mut self) {
        if self.selected_branch < self.branches.len() - 1 {
            self.selected_branch += 1;
            self.update_branch_visible_range();
        }
    }

    pub fn focus_next_panel(&mut self) {
        self.focused_panel = match self.focused_panel {
            Panel::CommitLog => Panel::Branches,
            Panel::Branches => Panel::CommitLog,
        };
    }

    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            if self.selected_index < self.visible_range.0 {
                self.scroll_up();
            }
        }
    }

    pub fn select_next(&mut self) {
        if self.selected_index < self.commit_log.len() - 1 {
            self.selected_index += 1;
            if self.selected_index >= self.visible_range.1 {
                self.scroll_down();
            }
        }
    }

    pub fn set_selected_commit_details(&mut self, details: String) {
        self.selected_commit_details = Some(details);
    }

    pub fn clear_selected_commit_details(&mut self) {
        self.selected_commit_details = None;
    }

    pub fn update_commit_details_visible_range(&mut self, chunk_height: usize) {
        let (start, _) = self.commit_details_visible_range;
        let end = usize::min(start + chunk_height, self.commit_details_total_lines);
        self.commit_details_visible_range = (start, end);
    }

    pub fn scroll_commit_details_up(&mut self, lines: usize) {
        let (start, end) = self.commit_details_visible_range;
        if start > 0 {
            let new_start = start.saturating_sub(lines);
            let new_end = usize::min(new_start + (end - start), self.commit_details_total_lines);
            self.commit_details_visible_range = (new_start, new_end);
        }
    }

    pub fn scroll_commit_details_down(&mut self, lines: usize) {
        let (start, end) = self.commit_details_visible_range;
        if end < self.commit_details_total_lines {
            let new_end = usize::min(end + lines, self.commit_details_total_lines);
            let new_start = new_end.saturating_sub(end - start);
            self.commit_details_visible_range = (new_start, new_end);
        }
    }

    pub fn jump_commit_log_up(&mut self) {
        let page_size = self.visible_count; // Number of items visible per page
        if self.selected_index > 0 {
            self.selected_index = self.selected_index.saturating_sub(page_size);
            self.update_visible_range();
        }
    }

    pub fn jump_commit_log_down(&mut self) {
        let page_size = self.visible_count; // Number of items visible per page
        if self.selected_index < self.commit_log.len() - 1 {
            self.selected_index =
                usize::min(self.selected_index + page_size, self.commit_log.len() - 1);
            self.update_visible_range();
        }
    }

    pub fn jump_branches_up(&mut self) {
        let page_size = self.branch_visible_count;
        if self.selected_branch > 0 {
            self.selected_branch = self.selected_branch.saturating_sub(page_size);
            self.update_branch_visible_range();
        }
    }

    pub fn jump_branches_down(&mut self) {
        let page_size = self.branch_visible_count;
        if self.selected_branch < self.branches.len() - 1 {
            self.selected_branch =
                usize::min(self.selected_branch + page_size, self.branches.len() - 1);
            self.update_branch_visible_range();
        }
    }
}
