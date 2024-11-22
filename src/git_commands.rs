use git2::Repository;

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
