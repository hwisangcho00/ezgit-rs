use crossterm::event::{Event, KeyCode};

pub enum Action {
    Quit,
    Refresh,
    NavigateUp,
    NavigateDown,
    Select,
    SwitchPanel,
    Deselect,
}

pub fn handle_user_input() -> Result<Option<Action>, std::io::Error> {
    if crossterm::event::poll(std::time::Duration::from_millis(200))? {
        if let Event::Key(key) = crossterm::event::read()? {
            return Ok(match key.code {
                KeyCode::Char('q') => Some(Action::Quit),
                KeyCode::Char('r') => Some(Action::Refresh),
                KeyCode::Up => Some(Action::NavigateUp),
                KeyCode::Down => Some(Action::NavigateDown),
                KeyCode::Enter => Some(Action::Select),
                KeyCode::Tab => Some(Action::SwitchPanel), // Switch between Commit Log and Branches
                KeyCode::Esc => Some(Action::Deselect),
                _ => None,
            });
        }
    }
    Ok(None)
}