use crate::app_state::{AppState, Panel, UIState, CommitState};
use crate::{git_commands, input};
use log::debug;

pub fn handle_event(app_state: &mut AppState) -> Result<bool, std::io::Error> {
    match input::handle_user_input()? {
        Some(input::Action::Quit) => return Ok(true),
        Some(input::Action::NavigateUp) => match app_state.focused_panel {
            Panel::CommitLog => app_state.select_previous(),
            Panel::Branches => app_state.select_previous_branch(),
        },
        Some(input::Action::NavigateDown) => match app_state.focused_panel {
            Panel::CommitLog => app_state.select_next(),
            Panel::Branches => app_state.select_next_branch(),
        },
        Some(input::Action::Select) => match app_state.ui_state {
            UIState::CommitMessage => {
                app_state.ui_state = UIState::ConfirmCommit;
            },
            UIState::ConfirmCommit => {
                if let Some(commit_state) = &app_state.commit_state {
                    match git_commands::commit_and_push(".", &commit_state.message) {
                        Ok(_) => {
                            app_state.ui_state = UIState::Normal;
                            app_state.commit_log = git_commands::get_commit_log("."); // Refresh commit log
                            println!("Changes committed and pushed successfully.");
                        }
                        Err(err) => {
                            eprintln!("Error during commit and push: {}", err);
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
            }
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
        Some(input::Action::Deselect) => {
            app_state.clear_selected_commit_details();
        },
        Some(input::Action::CommitWork) => {
            app_state.ui_state = UIState::CommitMessage;
            app_state.commit_state = Some(CommitState { message: String::new() });
        }

        _ => {}
    }
    Ok(false)
}