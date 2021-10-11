//! Display a list of currently active Pull Requests
//!
//! By "currently active", we mean "not yet deleted from the remote".
use libgitpr;

fn main() -> Result<(),libgitpr::GitError> {
    let git = libgitpr::Git::new();
    git.fetch_prune()?;
    let branches = git.all_branches()?;

    for pr_name in libgitpr::extract_pr_names(&branches) {
        println!("{}", pr_name);
    }
    Ok(())
}
