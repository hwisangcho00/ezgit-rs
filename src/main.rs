use std::io;
use crossterm::{execute, terminal, ExecutableCommand};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::style::{Style, Color};
use ezgit_rs::git_commands;
use ezgit_rs::app_state::{AppState, Panel, UIState};
use ezgit_rs::events::handle_event;
use ezgit_rs::logger::Logger;
use log::info;

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
                        Constraint::Percentage(30), // Branch List
                        Constraint::Percentage(20), // Input Prompt
                    ])
                    .split(f.area());
    
    
                    // Style for focused and unfocused panels
                    let focused_style = Style::default().fg(Color::Yellow);
                    let unfocused_style = Style::default();
        
                    // Render Commit Log
                    let commit_chunk_height = chunks[0].height as usize; // Height of the commit log chunk
                    app_state.visible_count = commit_chunk_height;       // Update visible_count dynamically
                    app_state.update_visible_range();                   // Update visible range based on selected index

                    // Render Commit Log
                    let visible_commits = &app_state.commit_log[app_state.visible_range.0..app_state.visible_range.1];
                    let commit_items: Vec<ListItem> = visible_commits
                        .iter()
                        .enumerate()
                        .map(|(i, commit)| {
                            let global_index = app_state.visible_range.0 + i;
                            if global_index == app_state.selected_index {
                                ListItem::new(commit.clone())
                                    .style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow))
                            } else {
                                ListItem::new(commit.clone())
                            }
                        })
                        .collect();

                    let commit_list = List::new(commit_items).block(
                        Block::default()
                            .title("Commit Log")
                            .borders(Borders::ALL)
                            .border_style(
                                if matches!(app_state.focused_panel, Panel::CommitLog) {
                                focused_style
                            } else {
                                unfocused_style
                            }),
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
        
        
                    let key_guide_text = vec![
                        "  - q: Quit the application (requires confirmation)",
                        "  - Esc: Cancel current action or return to the previous screen",
                        "  - Enter: Select item, confirm action, or proceed",
                        "  - Tab: Switch between Commit Log and Branches panel",
                        "  - ↑/↓: Navigate through items in the current panel",
                        "  - c: Start the commit workflow to add, commit, and push changes",
                        "  - b: Create and switch to a new branch",
                        "  - r: Refresh the Commit Log and Branches list",
                    ];
                    
                    let key_guide = Paragraph::new(key_guide_text.join("\n"))
                        .block(Block::default().title("Key Guide").borders(Borders::ALL))
                        .wrap(Wrap { trim: false });
                    
                    f.render_widget(key_guide, chunks[2]);
                    
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
                        .split(f.area());
                
                    let quit_prompt = Block::default()
                        .title("Are you sure you want to quit? (Press Enter to Confirm, Esc to Cancel)")
                        .borders(Borders::ALL);
                
                    f.render_widget(quit_prompt, chunks[0]);
                },
                UIState::CommitDetails => {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(1)
                        .constraints([Constraint::Percentage(100)])
                        .split(f.area());
                
                    let details = app_state.selected_commit_details.as_deref().unwrap_or("No details available");
                
                    let details_widget = Block::default()
                        .title("Commit Details (Press Esc to Return)")
                        .borders(Borders::ALL);
                
                    let details_paragraph = ratatui::widgets::Paragraph::new(details)
                        .block(details_widget)
                        .wrap(ratatui::widgets::Wrap { trim: false });
                
                    f.render_widget(details_paragraph, chunks[0]);
                },
                UIState::CreateBranch => {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(1)
                        .constraints([Constraint::Percentage(100)])
                        .split(f.area());
                
                    let input = app_state.branch_name.clone();
                    let prompt = format!("Enter new branch name: {}", input);
                
                    let branch_prompt = Paragraph::new(prompt)
                        .block(Block::default().title("Create Branch").borders(Borders::ALL));
                
                    f.render_widget(branch_prompt, chunks[0]);
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
