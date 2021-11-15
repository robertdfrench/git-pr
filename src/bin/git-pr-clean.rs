//! Remove local branches which have been merged into 'trunk'
use libgitpr;

fn main() -> Result<(),libgitpr::GitError> {
    let git = libgitpr::Git::new();
    let merged_branches = git.merged_branches()?;

    for branch in merged_branches.filter(|b| !b.is_head) {
        git.delete_branch(&branch.name.value)?;
    }

    Ok(())
}
