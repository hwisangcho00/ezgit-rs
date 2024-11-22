use std::io;
use crossterm::{event, execute, terminal, ExecutableCommand};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders};

fn main() -> Result<(), io::Error> {
    // Setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main event loop
    loop {
        // Draw UI
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Percentage(70), // Commit Log
                    Constraint::Percentage(20), // Branch List
                    Constraint::Percentage(10), // Input Prompt
                ])
                .split(f.size());

            let commit_log = Block::default()
                .title("Commit Log")
                .borders(Borders::ALL);
            f.render_widget(commit_log, chunks[0]);

            let branch_list = Block::default()
                .title("Branches")
                .borders(Borders::ALL);
            f.render_widget(branch_list, chunks[1]);

            let input_prompt = Block::default()
                .title("Input Prompt")
                .borders(Borders::ALL);
            f.render_widget(input_prompt, chunks[2]);
        })?;

        // Handle input (Quit with 'q')
        if crossterm::event::poll(std::time::Duration::from_millis(200))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                if let crossterm::event::KeyCode::Char('q') = key.code {
                    break;
                }
            }
        }
    }

    // Restore terminal
    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
    Ok(())
}
