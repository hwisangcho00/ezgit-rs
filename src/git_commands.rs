use git2::{BranchType, Repository};

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

