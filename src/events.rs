use crate::app_state::{AppState, Panel, UIState, CommitState, InputMode};
use crate::{git_commands, input};
use log::debug;

pub fn handle_event(app_state: &mut AppState) -> Result<bool, std::io::Error> {
    match app_state.input_mode {
        InputMode::Command => handle_command_mode(app_state),
        InputMode::Text => handle_text_mode(app_state),
    }
}

fn handle_text_mode(app_state: &mut AppState) -> Result<bool, std::io::Error> {
    match input::handle_user_input(true)? {
        Some(input::Action::TextInput(c)) => {
            if let Some(commit_state) = &mut app_state.commit_state {
                commit_state.message.push(c);
            }
        }
        Some(input::Action::Backspace) => {
            if let Some(commit_state) = &mut app_state.commit_state {
                commit_state.message.pop();
            }
        }
        Some(input::Action::Confirm) => {
            match app_state.ui_state {
                UIState::CommitMessage => {
                    // Transition to ConfirmCommit state after entering commit message
                    if let Some(commit_state) = &app_state.commit_state {
                        if commit_state.message.trim().is_empty() {
                            debug!("Cannot confirm: commit message is empty");
                        } else {
                            app_state.ui_state = UIState::ConfirmCommit; // Move to confirmation state
                            debug!("Transitioned to ConfirmCommit state");
                        }
                    }
                }
                UIState::ConfirmCommit => {
                    // Perform the commit and push operation
                    if let Some(commit_state) = &app_state.commit_state {
                        match git_commands::commit_and_push(".", &commit_state.message) {
                            Ok(_) => {
                                app_state.ui_state = UIState::Normal; // Reset to normal state
                                app_state.input_mode = InputMode::Command; // Back to command mode
                                app_state.commit_log = git_commands::get_commit_log("."); // Refresh commit log
                                debug!("Changes committed and pushed successfully.");
                            }
                            Err(err) => {
                                debug!("Error during commit and push: {}", err);
                                app_state.ui_state = UIState::Normal; // Reset on failure
                                app_state.input_mode = InputMode::Command;
                            }
                        }
                    }
                }
                _ => {
                    debug!("Confirm action ignored in current UIState");
                }
            }
        },
        
        Some(input::Action::Cancel) => {
            app_state.ui_state = UIState::Normal;
            app_state.commit_state = None;
            app_state.input_mode = InputMode::Command; // Switch back to Command Mode
        }
        _ => {}
    }
    Ok(false)
}

pub fn handle_command_mode(app_state: &mut AppState) -> Result<bool, std::io::Error> {
    match input::handle_user_input(false)? {
        Some(input::Action::Quit) => {
            match app_state.ui_state {
                UIState::Normal => {
                    app_state.ui_state = UIState::ConfirmQuit; // Transition to ConfirmQuit state
                    debug!("Transitioned to ConfirmQuit state");
                },
                _ => debug!("Quit action ignored in current UIState"),
            }
        },
        Some(input::Action::NavigateUp) => match app_state.focused_panel {
            Panel::CommitLog => app_state.select_previous(),
            Panel::Branches => app_state.select_previous_branch(),
        },
        Some(input::Action::NavigateDown) => match app_state.focused_panel {
            Panel::CommitLog => app_state.select_next(),
            Panel::Branches => app_state.select_next_branch(),
        },
        Some(input::Action::Select) => match app_state.ui_state {
            UIState::ConfirmQuit => {
                return Ok(true); // Exit the program
            }
            UIState::CommitMessage => {
                app_state.ui_state = UIState::ConfirmCommit;
            },
            UIState::ConfirmCommit => {
                if let Some(commit_state) = &app_state.commit_state {
                    match git_commands::commit_and_push(".", &commit_state.message) {
                        Ok(_) => {
                            app_state.ui_state = UIState::Normal;
                            app_state.commit_log = git_commands::get_commit_log("."); // Refresh commit log
                            debug!("Changes committed and pushed successfully.");
                        }
                        Err(err) => {
                            debug!("Error during commit and push: {}", err);
                            app_state.ui_state = UIState::Normal;
                        }
                    }
                }
            },
            UIState::Normal => {
                match app_state.focused_panel {

                    Panel::CommitLog => {
                        let selected_commit = &app_state.commit_log[app_state.selected_index];
                        
                        let selected_commit_hash = &selected_commit.split(":").next().unwrap_or("");
        
                        match git_commands::get_commit_details(".", selected_commit_hash) {
                            Ok(details) => {
                                app_state.set_selected_commit_details(details);
                            },
                            Err(err) => {
                                app_state.set_selected_commit_details(err);
                            }
        
                        }
                    }
                    Panel::Branches => {
                        let selected_branch = &app_state.branches[app_state.selected_branch];
                        if let Err(e) = crate::git_commands::checkout_branch(".", selected_branch) {
                            debug!("Error: {}", e);
                        } else {
                            app_state.commit_log = crate::git_commands::get_commit_log(".");
                        }
                    }
                }
            },
            
        },
        Some(input::Action::SwitchPanel) => {
            app_state.focus_next_panel();
        }
        Some(input::Action::Refresh) => {
            app_state.commit_log = crate::git_commands::get_commit_log(".");
            app_state.branches = crate::git_commands::get_branches(".");
            app_state.selected_index = 0;
            app_state.selected_branch = 0;
        },
        // Handle Deselect (Esc key) for canceling actions
        Some(input::Action::Deselect) => match app_state.ui_state {
            UIState::ConfirmQuit => {
                app_state.ui_state = UIState::Normal; // Cancel quit and return to normal state
                debug!("Quit cancelled");
            }
            UIState::CommitMessage | UIState::ConfirmCommit => {
                app_state.ui_state = UIState::Normal; // Cancel workflow
                app_state.commit_state = None;
                debug!("Workflow cancelled");
            }
            _ => {}
        },

        Some(input::Action::CommitWork) => {
            app_state.ui_state = UIState::CommitMessage;
            app_state.commit_state = Some(CommitState { message: String::new() });
            app_state.input_mode = InputMode::Text;
        }

        _ => {}
    }
    Ok(false)
}