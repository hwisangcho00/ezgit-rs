pub enum Panel {
    CommitLog,
    Branches,
}

pub enum UIState {
    Normal,
    CommitMessage,
    ConfirmCommit,
    ConfirmQuit,

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
    pub branches: Vec<String>,       // Branch list
    pub selected_branch: usize,      // Selected branch index
    pub focused_panel: Panel,        // Currently focused panel
    pub selected_commit_details: Option<String>,
    pub ui_state: UIState,
    pub commit_state: Option<CommitState>,
    pub input_mode: InputMode,
}

impl AppState {
    pub fn new(commit_log: Vec<String>, branches: Vec<String>) -> Self {
        Self {
            selected_index: 0,
            commit_log,
            branches,
            selected_branch: 0,
            focused_panel: Panel::CommitLog,
            selected_commit_details: None,
            ui_state: UIState::Normal,
            commit_state: None,
            input_mode: InputMode::Command,
        }
    }

    pub fn focus_next_panel(&mut self) {
        self.focused_panel = match self.focused_panel {
            Panel::CommitLog => Panel::Branches,
            Panel::Branches => Panel::CommitLog,
        };
    }

    // Move selection up
    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    // Move selection down
    pub fn select_next(&mut self) {
        if self.selected_index < self.commit_log.len() - 1 {
            self.selected_index += 1;
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