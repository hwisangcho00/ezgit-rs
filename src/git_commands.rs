use git2::{BranchType, Cred, MergeOptions, PushOptions, RemoteCallbacks, Repository, Signature};

use log::debug;

use std::env;
use dotenv::dotenv;



pub fn get_commit_log(repo_path: &str) -> Vec<String> {
    let repo = Repository::open(repo_path).expect("Failed to open repository");
    let mut revwalk = repo.revwalk().expect("Failed to create revwalk");
    revwalk.push_head().expect("Failed to push head");

    revwalk
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

    // Step 1: Find and checkout the branch
    let (object, reference) = repo
        .revparse_ext(branch_name)
        .map_err(|e| format!("Failed to find branch '{}' : {}", branch_name, e))?;

    repo.checkout_tree(&object, None)
        .map_err(|e| format!("Failed to checkout tree: {}", e))?;

    if let Some(ref_name) = reference.map(|r| r.name().unwrap_or("").to_string()) {
        repo.set_head(&ref_name)
            .map_err(|e| format!("Failed to set HEAD: {}", e))?;
    }

    // Step 2: Push the branch to the remote and set upstream
    let mut remote = repo.find_remote("origin").map_err(|e| format!("Failed to find remote: {}", e))?;
    let mut callbacks = git2::RemoteCallbacks::new();

    // Load credentials for authentication
    dotenv::dotenv().ok();
    let username = std::env::var("GIT_USERNAME").map_err(|_| "GIT_USERNAME not set".to_string())?;
    let token = std::env::var("GIT_PASSWORD").map_err(|_| "GIT_PASSWORD not set".to_string())?;

    callbacks.credentials(move |_url, username_from_url, _allowed_types| {
        git2::Cred::userpass_plaintext(
            username_from_url.unwrap_or(&username),
            &token,
        )
    });

    let mut push_options = git2::PushOptions::new();
    push_options.remote_callbacks(callbacks);

    let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
    remote
        .push(&[refspec], Some(&mut push_options))
        .map_err(|e| format!("Failed to push branch to remote: {}", e))?;

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

pub fn create_and_switch_branch(repo_path: &str, branch_name: &str) -> Result<(), String> {
    let repo = Repository::open(repo_path).map_err(|e| format!("Failed to open repository: {}", e))?;

    // Ensure branch name is valid
    if branch_name.trim().is_empty() {
        return Err("Branch name cannot be empty".to_string());
    }

    // Get the current HEAD commit
    let head_commit = repo.head()
        .and_then(|head| head.peel_to_commit())
        .map_err(|e| format!("Failed to get HEAD commit: {}", e))?;

    // Create a new branch
    repo.branch(branch_name, &head_commit, false)
        .map_err(|e| format!("Failed to create branch: {}", e))?;

    // Switch to the new branch
    let mut checkout_builder = git2::build::CheckoutBuilder::new();
    repo.set_head(&format!("refs/heads/{}", branch_name))
        .and_then(|_| repo.checkout_head(Some(&mut checkout_builder)))
        .map_err(|e| format!("Failed to switch to branch: {}", e))?;

    Ok(())
}

pub fn merge_into_branch(repo_path: &str, target_branch: &str) -> Result<(), String> {
    let repo = Repository::open(repo_path).map_err(|e| format!("Failed to open repository: {}", e))?;

    // Step 1: Check for uncommitted changes before switching branches
    let statuses = repo.statuses(None).map_err(|e| format!("Failed to get repository statuses: {}", e))?;
    if !statuses.is_empty() {
        // Auto-commit uncommitted changes
        let mut index = repo.index().map_err(|e| format!("Failed to get repository index: {}", e))?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
            .map_err(|e| format!("Failed to stage changes: {}", e))?;
        index.write().map_err(|e| format!("Failed to write index: {}", e))?;

        let oid = index.write_tree().map_err(|e| format!("Failed to write tree: {}", e))?;
        let tree = repo.find_tree(oid).map_err(|e| format!("Failed to find tree: {}", e))?;
        let head = repo.head().map_err(|e| format!("Failed to get HEAD: {}", e))?;
        let parent_commit = head.peel_to_commit().map_err(|e| format!("Failed to get parent commit: {}", e))?;
        let signature = repo.signature().map_err(|e| format!("Failed to create signature: {}", e))?;
        repo.commit(Some("HEAD"), &signature, &signature, "Auto-commit changes before merge", &tree, &[&parent_commit])
            .map_err(|e| format!("Failed to commit changes: {}", e))?;
    }

    // Step 2: Switch to the main branch
    repo.set_head(&format!("refs/heads/{}", target_branch))
        .map_err(|e| format!("Failed to set HEAD to target branch '{}': {}", target_branch, e))?;
    repo.checkout_head(None)
        .map_err(|e| format!("Failed to checkout target branch '{}': {}", target_branch, e))?;

    // Step 3: Get the current branch's HEAD commit
    let head_ref = repo.head().map_err(|e| format!("Failed to get HEAD: {}", e))?;
    let current_branch_commit = head_ref.peel_to_commit()
        .map_err(|e| format!("Failed to get commit for current branch: {}", e))?;

    // Step 4: Merge the current branch into the target branch
    let annotated_commit = repo.find_annotated_commit(current_branch_commit.id())
        .map_err(|e| format!("Failed to create annotated commit: {}", e))?;
    let mut merge_options = MergeOptions::new();
    repo.merge(&[&annotated_commit], Some(&mut merge_options), None)
        .map_err(|e| format!("Merge failed: {}", e))?;

    // Step 5: Check for conflicts
    if repo.index().map_err(|e| e.to_string())?.has_conflicts() {
        return Err("Merge completed with conflicts. Please resolve them manually.".to_string());
    }

    // Step 6: Commit the merge
    let signature = Signature::now("Merge Bot", "merge@example.com")
        .map_err(|e| format!("Failed to create signature: {}", e))?;
    let tree_oid = repo.index().map_err(|e| e.to_string())?.write_tree()
        .map_err(|e| format!("Failed to write tree: {}", e))?;
    let tree = repo.find_tree(tree_oid).map_err(|e| format!("Failed to find tree: {}", e))?;
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Merge changes into main branch",
        &tree,
        &[&current_branch_commit],
    ).map_err(|e| format!("Failed to commit merge: {}", e))?;

    debug!("Merge completed successfully. You are now on the '{}' branch.", target_branch);

    debug!("Is this in?");

    Ok(())
}

// Will this work?

