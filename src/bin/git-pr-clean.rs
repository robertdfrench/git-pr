//! Remove local branches which have been merged into 'trunk'
use libgitpr;

fn main() -> Result<(),libgitpr::GitError> {
    let git = libgitpr::Git::new();
    let merged_branches = git.merged_branches()?;

    for branch in libgitpr::extract_deletable_branches(&merged_branches) {
        git.delete_branch(&branch)?;
    }

    Ok(())
}
