use crate::app_state::{AppState, Panel};
use crate::input;

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
        Some(input::Action::Select) => match app_state.focused_panel {
            Panel::CommitLog => {
                println!(
                    "Selected commit: {}",
                    app_state.commit_log[app_state.selected_index]
                );
            }
            Panel::Branches => {
                let selected_branch = &app_state.branches[app_state.selected_branch];
                if let Err(e) = crate::git_commands::checkout_branch(".", selected_branch) {
                    println!("Error: {}", e);
                } else {
                    app_state.commit_log = crate::git_commands::get_commit_log(".");
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
        }
        _ => {}
    }
    Ok(false)
}