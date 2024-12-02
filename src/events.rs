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
        Some(input::Action::TextInput(c)) => match app_state.ui_state {
            UIState::CommitMessage => {
                if let Some(commit_state) = &mut app_state.commit_state {
                    commit_state.message.push(c); // Add character to commit message
                }
            }
            UIState::CreateBranch => {
                app_state.branch_name.push(c); // Add character to branch name
            }
            _ => {}
        },
        Some(input::Action::Backspace) => match app_state.ui_state {
            UIState::CommitMessage => {
                if let Some(commit_state) = &mut app_state.commit_state {
                    commit_state.message.pop(); // Remove last character from commit message
                }
            }
            UIState::CreateBranch => {
                app_state.branch_name.pop(); // Remove last character from branch name
            }
            _ => {}
        },
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
                },
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
                },
                UIState::CreateBranch => {
                    if let Err(err) = git_commands::create_and_switch_branch(".", &app_state.branch_name) {
                        debug!("Error creating branch: {}", err);
                    } else {
                        app_state.branches = git_commands::get_branches("."); // Refresh branch list
                        debug!("Branch '{}' created and switched successfully", app_state.branch_name);
                    }
                    app_state.ui_state = UIState::Normal; // Return to normal state
                    app_state.input_mode = InputMode::Command;
                },
                _ => {
                    debug!("Confirm action ignored in current UIState");
                }
            }
        },
        
        Some(input::Action::Cancel) => {
            app_state.ui_state = UIState::Normal;
            app_state.commit_state = None;
            app_state.branch_name = String::new();
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
        Some(input::Action::NavigateUp) => {
            match app_state.ui_state {
                UIState::CommitDetails => {
                    // Scroll up in commit details
                    app_state.scroll_commit_details_up(1);
                    debug!("Scrolled up in commit details");
                }
                UIState::Normal => match app_state.focused_panel {
                    Panel::CommitLog => app_state.select_previous(),
                    Panel::Branches => app_state.select_previous_branch(),
                },
                _ => {}
            }
        },
        Some(input::Action::NavigateDown) => {
            match app_state.ui_state {
                UIState::CommitDetails => {
                    // Scroll down in commit details
                    app_state.scroll_commit_details_down(1);
                    debug!("Scrolled down in commit details");
                }
                UIState::Normal => match app_state.focused_panel {
                    Panel::CommitLog => app_state.select_next(),
                    Panel::Branches => app_state.select_next_branch(),
                },
                _ => {}
            }
        },
        Some(input::Action::NavigateLeft) => {
            if matches!(app_state.focused_panel, Panel::CommitLog) && app_state.horizontal_offset > 0 {
                app_state.horizontal_offset -= 1;
            }
        },
        Some(input::Action::NavigateRight) => {
            if matches!(app_state.focused_panel, Panel::CommitLog) {
                app_state.horizontal_offset += 1; // Increase the offset to scroll right
            }
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
                        let commit_hash = selected_commit.split(" | ").next().unwrap_or("");
        
                        match git_commands::get_commit_details(".", commit_hash) {
                            Ok(details) => {
                                app_state.set_selected_commit_details(details);
                                app_state.ui_state = UIState::CommitDetails; // Transition to CommitDetails state
                                debug!("Showing commit details");
                            },
                            Err(err) => {
                                debug!("Error fetching commit details: {}", err);
                            }
        
                        }
                    }
                    Panel::Branches => {
                        let selected_branch = app_state.branches[app_state.selected_branch].clone();
                        match crate::git_commands::checkout_branch(".", &selected_branch) {
                            Ok(_) => {
                                app_state.commit_log = crate::git_commands::get_commit_log("."); // Refresh commit log
                                app_state.branches = crate::git_commands::get_branches(".");     // Refresh branch list
                                app_state.branch_name = selected_branch.clone();                // Update the current branch
                                debug!("Switched to branch: {}", selected_branch);
                            }
                            Err(err) => {
                                debug!("Error checking out branch: {}", err);
                                app_state.ui_state = UIState::Error; // Transition to error state
                                app_state.error_message = Some(err); // Store error message for display
                            }
                        }
                    }
                }
            },
            UIState::ConfirmMerge => {
                // Determine the target branch (main or master)
                let target_branch = if app_state.branches.contains(&"main".to_string()) {
                    "main"
                } else {
                    "master"
                };
            
                // Attempt to merge into the target branch
                match git_commands::merge_into_branch(".", target_branch) {
                    Ok(_) => {
                        debug!("Successfully merged the current branch into '{}'", target_branch);
                        app_state.commit_log = git_commands::get_commit_log("."); // Refresh commit log
                        app_state.ui_state = UIState::Normal; // Return to normal state after merging
                        app_state.error_message = None; // Clear any previous error messages
                    }
                    Err(err) => {
                        debug!("Error merging into '{}': {}", target_branch, err);
                        app_state.ui_state = UIState::Error; // Transition to error state
                        app_state.error_message = Some(err); // Store the error message
                    }
                }
            },
            
            _ => {}
            
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
            },
             UIState::CommitDetails => {
                app_state.ui_state = UIState::Normal; // Return to normal state
                debug!("Exited commit details view");
             },
             UIState::KeyGuide => {
                app_state.ui_state = UIState::Normal; // Return to normal state
                debug!("Exited key guide view");
             },
             UIState::ConfirmMerge => {
                app_state.ui_state = UIState::Normal;
             },
             UIState::Error => {
                app_state.ui_state = UIState::Normal; // Return to Normal state
                app_state.error_message = None;      // Clear the error message
             },
            _ => {}
        },

        Some(input::Action::CommitWork) => {
            if app_state.ui_state == UIState::Normal {
                app_state.ui_state = UIState::CommitMessage;
                app_state.commit_state = Some(CommitState { message: String::new() });
                app_state.input_mode = InputMode::Text;
            }

        },

        Some(input::Action::CreateBranch) => {
            if app_state.ui_state == UIState::Normal {
                app_state.ui_state = UIState::CreateBranch;
                app_state.input_mode = InputMode::Text;
                app_state.branch_name = String::new();
            }
        },
        Some(input::Action::ShowKeyGuide) => {
            app_state.ui_state = UIState::KeyGuide;
        },
        Some(input::Action::MergeBranch) => {
            app_state.ui_state = UIState::ConfirmMerge;
        }

        _ => {}
    }
    Ok(false)
}