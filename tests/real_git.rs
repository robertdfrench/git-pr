//! Test the git "client" wrapper against the real git binary.
use libgitpr::Git;
use std::process::Command;
use std::process::Stdio;
use tempdir::TempDir;

// Implementing this above produces a warning, since the function is (by design) never used by
// other application code. Since it is only used in this module, we implement this function
// local to this module, thus eliminating the dead code warning.
fn temp_repo() -> Git {
    let working_dir = Box::new(TempDir::new("git-pr").unwrap());

    // git init in new unique dir
    let status = Command::new("git")
        .stdout(Stdio::null())
        .arg("-C").arg(working_dir.as_ref().as_ref())
        .args(&["init"]).status().unwrap();
    assert!(status.success());

    // Setup git config for email
    let status = Command::new("git")
        .arg("-C").arg(working_dir.as_ref().as_ref())
        .args(&["config","user.email","you@example.com"]).status().unwrap();
    assert!(status.success());

    // Setup git config for name
    let status = Command::new("git")
        .arg("-C").arg(working_dir.as_ref().as_ref())
        .args(&["config","user.name","Your Name"]).status().unwrap();
    assert!(status.success());

    // create trunk branch
    let status = Command::new("git")
        .arg("-C").arg(working_dir.as_ref().as_ref())
        .args(&["checkout","-b","trunk"]).status().unwrap();
    assert!(status.success());

    // empty commit to actually create trunk branch
    let status = Command::new("git")
        .arg("-C").arg(working_dir.as_ref().as_ref())
        .args(&["commit","--allow-empty","-m","hello"]).status().unwrap();
    assert!(status.success());

    // create a fake branch to test deletion
    let status = Command::new("git")
        .arg("-C").arg(working_dir.as_ref().as_ref())
        .args(&["branch","hotfix"]).status().unwrap();
    assert!(status.success());

    Git{ program: "git".to_string(), working_dir }
}


#[test]
fn version() {
    let git = temp_repo();
    let version = git.version().unwrap();
    assert!(version.starts_with("git version 2"));
}

#[test]
fn fetch_and_prune() {
    let git = temp_repo();
    git.fetch_prune().unwrap();
}

#[test]
fn can_list_all_branches() {
    let git = temp_repo();
    let branches = git.all_branches().unwrap();
    assert!(branches.value.contains("trunk"));
}

// Cleaning PRs requires that we identify "old" branches (those which have been merged into trunk),
// and that we delete those branches. Because the tests run in parallel, we need to ensure that our
// check for the existence of the "hotfix" branch always happens *before* our attempt to delete the
// "hotfix" branch. So this test case exercises all the git client functionality we would need in
// order to implement the "pr-clean" subcommand.
#[test]
fn could_clean() {
    let git = temp_repo();
    let branches = git.merged_branches().unwrap();
    assert!(branches.value.contains("hotfix"));

    git.delete_branch("hotfix").unwrap();
    let branches = git.all_branches().unwrap();
    assert!(!branches.value.contains("hotfix"));
}

#[test]
fn can_get_hash_of_head() {
    // The hash will change every time, but this is one of the few git commands for which we can
    // know the exact length of the output. Weak, but best we can do until we add more capabilities
    // to the client.
    let git = temp_repo();
    let hash = git.rev_parse_head().unwrap();
    assert_eq!(hash.len(), 7);
}

#[test]
fn can_create_new_branch() {
    // Show that we can create a new branch in this repo, and verify its existence by querying the
    // list of branches and showing that this new branch is among them.
    let git = temp_repo();
    git.create_branch("knurt").unwrap();
    let branches = git.all_branches().unwrap();
    assert!(branches.value.contains("knurt"));
}
