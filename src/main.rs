use std::io;
use crossterm::{execute, terminal, ExecutableCommand};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ezgit_rs::{git_commands, input};

pub enum Panel {
    CommitLog,
    Branches,
}

pub struct AppState {
    pub selected_index: usize,       // Selected commit index
    pub commit_log: Vec<String>,     // Commit log
    pub branches: Vec<String>,       // Branch list
    pub selected_branch: usize,      // Selected branch index
    pub focused_panel: Panel,        // Currently focused panel
}



impl AppState {
    pub fn new(commit_log: Vec<String>, branches: Vec<String>) -> Self {
        Self {
            selected_index: 0,
            commit_log,
            branches,
            selected_branch: 0,
            focused_panel: Panel::CommitLog,
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
}

fn main() -> Result<(), io::Error> {
    // Setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let initial_commit_log = git_commands::get_commit_log(".");
    let initial_branch = git_commands::get_branches(".");
    let mut app_state = AppState::new(initial_commit_log, initial_branch);

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

            // Render Commit Log
            let commit_items: Vec<ListItem> = app_state
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
            let commit_list = List::new(commit_items)
            .block(Block::default().title("Commit Log").borders(Borders::ALL));
            f.render_widget(commit_list, chunks[0]);


            // Render Branch List
            let branch_items: Vec<ListItem> = app_state
                .branches
                .iter()
                .enumerate()
                .map(|(i, branch)| {
                    if i == app_state.selected_branch {
                        ListItem::new(branch.clone())
                            .style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan))
                    } else {
                        ListItem::new(branch.clone())
                    }
                })
                .collect();
            let branch_list = List::new(branch_items)
                .block(Block::default().title("Branches").borders(Borders::ALL));
            f.render_widget(branch_list, chunks[1]);


            let input_prompt = Block::default()
                .title("Input Prompt")
                .borders(Borders::ALL);
            f.render_widget(input_prompt, chunks[2]);
        })?;

        match input::handle_user_input()? {
            Some(input::Action::Quit) => break,
            Some(input::Action::NavigateUp) => match app_state.focused_panel {
                Panel::CommitLog => app_state.select_previous(),
                Panel::Branches => app_state.select_previous_branch(),
            },
            Some(input::Action::NavigateDown) => match app_state.focused_panel {
                Panel::CommitLog => app_state.select_next(),
                Panel::Branches => app_state.select_next_branch(),
            },
            Some(input::Action::Select) => match app_state.focused_panel {
                Panel::CommitLog => {
                    // Add commit-specific logic here, e.g., showing details or diffs
                    println!("Selected commit: {}", app_state.commit_log[app_state.selected_index]);
                }
                Panel::Branches => {
                    let selected_branch = &app_state.branches[app_state.selected_branch];
                    if let Err(e) = git_commands::checkout_branch(".", selected_branch) {
                        println!("Error: {}", e);
                    } else {
                        app_state.commit_log = git_commands::get_commit_log(".");
                    }
                }
            },
            Some(input::Action::SwitchPanel) => {
                app_state.focus_next_panel();
            }
            Some(input::Action::Refresh) => {
                app_state.commit_log = git_commands::get_commit_log(".");
                app_state.branches = git_commands::get_branches(".");
                app_state.selected_index = 0;
                app_state.selected_branch = 0;
            }
            _ => {}
        }
    }

    // Restore terminal
    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
    Ok(())
}
