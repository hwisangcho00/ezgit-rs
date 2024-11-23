use std::io;
use crossterm::{execute, terminal, ExecutableCommand};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ezgit_rs::{git_commands, input};

pub struct AppState {
    pub selected_index: usize,
    pub commit_log: Vec<String>,
}

impl AppState {
    pub fn new(commit_log: Vec<String>) -> Self {
        Self {
            selected_index: 0,
            commit_log,
        }
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
}

fn main() -> Result<(), io::Error> {
    // Setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let initial_commit_log = git_commands::get_commit_log(".");
    let mut app_state = AppState::new(initial_commit_log);

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

            let items: Vec<ListItem> = app_state
            .commit_log
            .iter()
            .enumerate()
            .map(|(i, commit)| {
                if i == app_state.selected_index {
                    ListItem::new(commit.clone())
                        .style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow))
                } else {
                    ListItem::new(commit.clone())
                }
            })
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
            Some(input::Action::NavigateUp) => app_state.select_previous(),
            Some(input::Action::NavigateDown) => app_state.select_next(),
            Some(input::Action::Refresh) => {
                app_state.commit_log = git_commands::get_commit_log(".");
                app_state.selected_index = 0; // Reset selection
            }
            _ => {}
        }
    }

    // Restore terminal
    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
    Ok(())
}
