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
    Command,  // Default mode for handling commands
    Text,     // Mode for handling text input
}

pub struct AppState {
    pub selected_index: usize,       // Selected commit index
    pub commit_log: Vec<String>,     // Commit log
    pub visible_range: (usize, usize), // Visible range of commits
    pub visible_count: usize,
    pub branches: Vec<String>,       // Branch list
    pub selected_branch: usize,      // Selected branch index
    pub focused_panel: Panel,        // Currently focused panel
    pub selected_commit_details: Option<String>,
    pub ui_state: UIState,
    pub commit_state: Option<CommitState>,
    pub input_mode: InputMode,
    pub branch_name: String,
    pub error_message: Option<String>,
}

impl AppState {

    pub fn new(commit_log: Vec<String>, branches: Vec<String>, repo_path: &str) -> Self {
        // Determine the current branch
        let current_branch = match Repository::open(repo_path)
            .and_then(|repo| repo.head().and_then(|head| head.shorthand().map(String::from).ok_or(git2::Error::from_str("No branch name"))))
        {
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
            branches,
            selected_branch,
            focused_panel: Panel::CommitLog,
            selected_commit_details: None,
            ui_state: UIState::Normal,
            commit_state: None,
            input_mode: InputMode::Command,
            branch_name: current_branch,
            error_message: None,
        }
    }

    pub fn update_visible_range(&mut self) {
        let start = self.selected_index.saturating_sub(self.selected_index % self.visible_count);
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

    pub fn select_previous_branch(&mut self) {
        if self.selected_branch > 0 {
            self.selected_branch -= 1;
        }
    }

    pub fn select_next_branch(&mut self) {
        if self.selected_branch < self.branches.len() - 1 {
            self.selected_branch += 1;
        }
    }

    pub fn set_selected_commit_details(&mut self, details: String) {
        self.selected_commit_details = Some(details);
    }

    pub fn clear_selected_commit_details(&mut self) {
        self.selected_commit_details = None;
    }

}