//! Display a list of currently active Pull Requests
use std::io;
use libgitpr;

fn main() -> io::Result<()> {
    let git = libgitpr::Git::new();
    git.fetch_prune()?;
    let branches = git.all_branches()?;

    for pr_name in libgitpr::extract_pr_names(&branches) {
        println!("{}", pr_name);
    }
    Ok(())
}
