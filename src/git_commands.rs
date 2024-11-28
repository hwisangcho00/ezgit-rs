use git2::{BranchType, Repository, Cred, PushOptions, RemoteCallbacks};

// use log::debug;

use std::env;
use dotenv::dotenv;



pub fn get_commit_log(repo_path: &str) -> Vec<String> {
    let repo = Repository::open(repo_path).expect("Failed to open repository");
    let mut revwalk = repo.revwalk().expect("Failed to create revwalk");
    revwalk.push_head().expect("Failed to push head");

    revwalk
        .take(10)
        .filter_map(|oid| oid.ok())
        .filter_map(|oid| repo.find_commit(oid).ok())
        .map(|commit| format!("{}: {}", commit.id(), commit.summary().unwrap_or("No message")))
        .collect()
}

pub fn get_branches(repo_path: &str) -> Vec<String> {
    let repo = Repository::open(repo_path).expect("Failed to open repository");

    repo.branches(Some(BranchType::Local))
        .expect("Failed to retrieve branches")
        .filter_map(|branch| branch.ok())
        .filter_map(|(branch, _)| branch.name().ok()?.map(String::from))  // Handle Option and convert to String
        .collect()
}

pub fn checkout_branch(repo_path: &str, branch_name: &str) -> Result<(), String> {

    let repo = Repository::open(repo_path).map_err(|e| e.to_string())?;

    let (object, reference) = repo
        .revparse_ext(branch_name)
        .map_err(|e| format!("Failed to find branch '{}' : {}", branch_name, e))?;

    repo.checkout_tree(&object, None)
        .map_err(|e| format!("Failed to checkout tree: {}", e))?;

    if let Some(ref_name) = reference.map(|r| r.name().unwrap_or("").to_string()) {
        repo.set_head(&ref_name)
            .map_err(|e| format!("Failed to set HEAD: {}", e))?;
    }

    Ok(())
}

pub fn get_commit_details(repo_path: &str, commit_hash: &str) -> Result<String, String> {
    let repo = Repository::open(repo_path).map_err(|e| e.to_string())?;
    let oid = repo.revparse_single(commit_hash).map_err(|e| e.to_string())?.id();
    let commit = repo.find_commit(oid).map_err(|e| e.to_string())?;

    let details = format!(
        "Commit Hash: {}\nAuthor: {} <{}>\nDate: {}\n\nMessage:\n{}",
        commit.id(),
        commit.author().name().unwrap_or("Unknown"),
        commit.author().email().unwrap_or("Unknown"),
        commit.time().seconds(),
        commit.message().unwrap_or("No message")
    );

    Ok(details)
}

pub fn commit_and_push(repo_path: &str, commit_message: &str) -> Result<(), String> {
    dotenv().ok();
    let username = env::var("GIT_USERNAME").map_err(|_| "GIT_USERNAME not set".to_string())?;
    let token = env::var("GIT_PASSWORD").map_err(|_| "GIT_PASSWORD not set".to_string())?; // Use the token here

    let repo = Repository::open(repo_path).map_err(|e| format!("Failed to open repository: {}", e))?;

    let mut index = repo.index().map_err(|e| format!("Failed to get repository index: {}", e))?;
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
        .map_err(|e| format!("Failed to stage changes: {}", e))?;
    index.write().map_err(|e| format!("Failed to write index: {}", e))?;

    let oid = index.write_tree().map_err(|e| format!("Failed to write tree: {}", e))?;
    let tree = repo.find_tree(oid).map_err(|e| format!("Failed to find tree: {}", e))?;
    let head = repo.head().map_err(|e| format!("Failed to get HEAD: {}", e))?;
    let parent_commit = head.peel_to_commit().map_err(|e| format!("Failed to get parent commit: {}", e))?;
    let signature = repo.signature().map_err(|e| format!("Failed to create signature: {}", e))?;
    repo.commit(Some("HEAD"), &signature, &signature, commit_message, &tree, &[&parent_commit])
        .map_err(|e| format!("Failed to commit changes: {}", e))?;

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |_url, username_from_url, _allowed_types| {
        Cred::userpass_plaintext(
            username_from_url.unwrap_or(&username), // Use username from URL or fallback
            &token,                                 // Use the PAT as the password
        )
    });

    let mut push_options = PushOptions::new();
    push_options.remote_callbacks(callbacks);

    let mut remote = repo.find_remote("origin").map_err(|e| format!("Failed to find remote: {}", e))?;
    remote.push(&["refs/heads/main:refs/heads/main"], Some(&mut push_options))
        .map_err(|e| format!("Failed to push changes: {}", e))?;

    Ok(())
}