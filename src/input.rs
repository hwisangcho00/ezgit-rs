use crossterm::event::{Event, KeyCode};

pub enum Action {
    Quit,
    Refresh,
    NavigateUp,
    NavigateDown,
    Select,
}

pub fn handle_user_input() -> Result<Option<Action>, std::io::Error> {
    if crossterm::event::poll(std::time::Duration::from_millis(200))? {
        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            return Ok(match key.code {
                crossterm::event::KeyCode::Char('q') => Some(Action::Quit),
                crossterm::event::KeyCode::Char('r') => Some(Action::Refresh),
                crossterm::event::KeyCode::Up => Some(Action::NavigateUp),
                crossterm::event::KeyCode::Down => Some(Action::NavigateDown),
                crossterm::event::KeyCode::Enter => Some(Action::Select),
                _ => None,
            });
        }
    }
    Ok(None)
}