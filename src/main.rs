use std::io;
use crossterm::{event, execute, terminal, ExecutableCommand};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ezgit_rs::{git_commands, input};

fn main() -> Result<(), io::Error> {
    // Setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;


    let mut commit_log = git_commands::get_commit_log(".");
    let mut selected_index = 0;
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
                .split(f.area());

            let items: Vec<ListItem> = commit_log
            .iter()
            .map(|commit| ListItem::new(commit.clone()))
            .collect();

            let commit_list = List::new(items)
                .block(Block::default().title("Commit Log").borders(Borders::ALL));

            f.render_widget(commit_list, chunks[0]);

            let branch_list = Block::default()
                .title("Branches")
                .borders(Borders::ALL);
            f.render_widget(branch_list, chunks[1]);

            let input_prompt = Block::default()
                .title("Input Prompt")
                .borders(Borders::ALL);
            f.render_widget(input_prompt, chunks[2]);
        })?;

        // Handle input
        match input::handle_user_input()? {
            Some(input::Action::Quit) => break, // Exit loop on 'q'
            Some(input::Action::Refresh) => {
                commit_log = git_commands::get_commit_log("."); // Refresh commit log
            }
            Some(input::Action::NavigateUp) => {
                if selected_index > 0 {
                    selected_index -= 1;
                }
            }
            Some(input::Action::NavigateDown) => {
                if selected_index < commit_log.len() - 1 {
                    selected_index += 1;
                }
            }
            Some(input::Action::Select) => {
                println!("Selected item: {}", commit_log[selected_index]);
            }
            None => {}
        }
    }

    // Restore terminal
    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
    Ok(())
}
