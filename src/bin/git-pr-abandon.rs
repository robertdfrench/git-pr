//! Abadon the given PR locally and remotely
use libgitpr;
use std::env::args;
use std::process::exit;

fn main() -> Result<(),libgitpr::GitError> {

    // We expect exactly one argument, a PR name.
    match args().nth(1).as_deref() {
        None => {
            eprintln!("A Pull Request name is required: git pr-abandon <name>");
            exit(1);
        },
        Some(name) => {
            let git = libgitpr::Git::new();
            git.fetch_prune()?; // needed?
            let mut status:Option<libgitpr::GitError> = None;

            // Delete remote branchs:
            let all_remote_branches = git.all_remote_branches()?;
            let remotes_to_delete = libgitpr::filter_remote_branches(name, &all_remote_branches);
            for branch in remotes_to_delete {
                if let Err(error) = git.push_delete(&branch) {
                    status = Some(error);
                }
            }

            // Delete local branchs:
            let local_branches = git.all_local_branches()?;
            let locals_to_delete = libgitpr::filter_local_branches(name, &local_branches);
            for branch in locals_to_delete {
                if let Err(error) = git.force_delete_branch(&branch) {
                    status = Some(error);
                }
            }

            // Did any deletes have problems?
            if let Some(_) = status {
                exit(1);
            }
        }
    }

    Ok(())
}
