//! Create a new local branch with an associated upstream tracking branch for a pull request.
//!
//! This tool currently assumes 'origin' will be the name of the remote.
use libgitpr;
use std::env::args;
use std::process::exit;


fn main() -> Result<(),libgitpr::GitError> {

    // We expect exactly one argument, a PR name.
    match args().nth(1).as_deref() {
        None => {
            eprintln!("A Pull Request name is required: git pr-create <name>");
            exit(1)
        },
        Some(name) => {
            let git = libgitpr::Git::new();

            // Find the current hash of HEAD, and create a new branch called "name/hash"
            let hash = git.rev_parse_head()?;
            let branch_name = format!("{}/{}",name,hash);
            git.create_branch(&branch_name)?;

            // Push that branch to the remote named *origin*
            git.push_upstream(&branch_name)?;
        }
    }

    Ok(())
}
