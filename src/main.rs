use std::io;
use crossterm::{execute, terminal, ExecutableCommand};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::style::{Style, Color};
use ezgit_rs::{git_commands, input};
use ezgit_rs::app_state::{AppState, Panel, UIState};
use ezgit_rs::events::handle_event;
use ezgit_rs::logger::Logger;
use log::{info, debug, error};

fn main() -> Result<(), io::Error> {
    // Initialize the logger
    Logger::init("debug.log", log::LevelFilter::Debug);
    info!("Logger initialized");

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

            match app_state.ui_state {
                UIState::Normal => {
                    let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([
                        Constraint::Percentage(50), // Commit Log
                        Constraint::Percentage(20), // Branch List
                        Constraint::Percentage(30), // Input Prompt
                    ])
                    .split(f.area());
    
    
                // Style for focused and unfocused panels
                let focused_style = Style::default().fg(Color::Yellow);
                let unfocused_style = Style::default();
    
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
                    .block(
                        Block::default()
                            .title("Commit Log")
                            .borders(Borders::ALL)
                            .border_style(
                                if matches!(app_state.focused_panel, Panel::CommitLog) {
                                    focused_style
                                } else {
                                    unfocused_style
                                },
                            ),
                    );
    
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
    
                // Branch List
                let branch_list = List::new(branch_items)
                    .block(
                        Block::default()
                            .title("Branches")
                            .borders(Borders::ALL)
                            .border_style(
                                if matches!(app_state.focused_panel, Panel::Branches) {
                                    focused_style
                                } else {
                                    unfocused_style
                                },
                            ),
                    );
    
                f.render_widget(branch_list, chunks[1]);
    
    
                // Conditionally Render Input Prompt or Commit Details
                if let Some(details) = &app_state.selected_commit_details {
                    let commit_details = Paragraph::new(details.clone())
                        .block(Block::default().title("Commit Details").borders(Borders::ALL));
                    f.render_widget(commit_details, chunks[2]);
                } else {
                    let input_prompt = Block::default()
                        .title("Input Prompt")
                        .borders(Borders::ALL);
                    f.render_widget(input_prompt, chunks[2]);
                }
                    
                },

                UIState::CommitMessage => {
                    // Render UI for entering commit message
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(1)
                        .constraints([Constraint::Percentage(100)]) // Full area for input
                        .split(f.area());

                    // Get the current commit message from CommitState
                    let commit_message = app_state
                        .commit_state
                        .as_ref()
                        .map_or("", |state| &state.message);

                    // Create a paragraph to display the message
                    let input_prompt = Paragraph::new(commit_message)
                        .block(
                            Block::default()
                                .title("Enter Commit Message (Press Enter to Confirm, Esc to Cancel)")
                                .borders(Borders::ALL),
                        );

                    // Render the input prompt
                    f.render_widget(input_prompt, chunks[0]);
                }
                UIState::ConfirmCommit => {
                    // Render UI for confirming commit and push
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(1)
                        .constraints([Constraint::Percentage(100)])
                        .split(f.area());
                    let confirm_prompt = Block::default()
                        .title("Are you sure you want to commit and push? (Press Enter to Confirm)")
                        .borders(Borders::ALL);
                    f.render_widget(confirm_prompt, chunks[0]);
                },
                UIState::ConfirmQuit => {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(1)
                        .constraints([Constraint::Percentage(100)])
                        .split(f.size());
                
                    let quit_prompt = Block::default()
                        .title("Are you sure you want to quit? (Press Enter to Confirm, Esc to Cancel)")
                        .borders(Borders::ALL);
                
                    f.render_widget(quit_prompt, chunks[0]);
                }
            }
        })?;

        if handle_event(&mut app_state)? {
            break;
        }

    }

    // Restore terminal
    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
    Ok(())
}
