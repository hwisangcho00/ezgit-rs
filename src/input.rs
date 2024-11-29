use crossterm::event::{Event, KeyCode};

pub enum Action {
    Quit,
    Refresh,
    NavigateUp,
    NavigateDown,
    Select,
    SwitchPanel,
    Deselect,
    CommitWork,
    CreateBranch,
    ShowKeyGuide,
    MergeBranch,

    // Text input actions
    TextInput(char), // Character input for text mode
    Backspace,       // Handle backspace during text input
    Confirm,         // Confirm (e.g., Enter key) during text input
    Cancel,          // Cancel (e.g., Escape key) during text input
}

pub fn handle_user_input(is_text_mode: bool) -> Result<Option<Action>, std::io::Error> {
    if crossterm::event::poll(std::time::Duration::from_millis(200))? {
        if let Event::Key(key) = crossterm::event::read()? {
            if is_text_mode {
                // Handle input in text mode
                return Ok(match key.code {
                    KeyCode::Char(c) => Some(Action::TextInput(c)), // Capture character input
                    KeyCode::Backspace => Some(Action::Backspace),  // Handle backspace
                    KeyCode::Enter => Some(Action::Confirm),        // Confirm input
                    KeyCode::Esc => Some(Action::Cancel),           // Cancel text input
                    _ => None,
                });
            } else{
                return Ok(match key.code {
                    KeyCode::Char('q') => Some(Action::Quit),
                    KeyCode::Char('r') => Some(Action::Refresh),
                    KeyCode::Up => Some(Action::NavigateUp),
                    KeyCode::Down => Some(Action::NavigateDown),
                    KeyCode::Enter => Some(Action::Select),
                    KeyCode::Tab => Some(Action::SwitchPanel), // Switch between Commit Log and Branches
                    KeyCode::Esc => Some(Action::Deselect),
                    KeyCode::Char('c') => Some(Action::CommitWork),
                    KeyCode::Char('b') => Some(Action::CreateBranch),
                    KeyCode::Char('g') => Some(Action::ShowKeyGuide),
                    KeyCode::Char('m') => Some(Action::MergeBranch),
                    _ => None,
                });
            }
        }
    }
    Ok(None)
}