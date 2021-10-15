//! Test the git "client" wrapper against the real git binary.
use lazy_static::lazy_static;
use libgitpr::Git;
use std::env::set_current_dir;
use std::path::Path;
use std::process::Command;
use tempdir::TempDir;

struct TestState {
    working_dir: TempDir
}

impl TestState {
    fn new(prefix: &str) -> Self {
        let working_dir = TempDir::new(prefix).unwrap();
        set_current_dir(working_dir.path()).unwrap();

        // git init in new unique dir
        let status = Command::new("git").args(&["init"]).status().unwrap();
        assert!(status.success());

        // Setup git config for email
        let status = Command::new("git")
            .args(&["config","user.email","you@example.com"]).status().unwrap();
        assert!(status.success());

        // Setup git config for name
        let status = Command::new("git")
            .args(&["config","user.name","Your Name"]).status().unwrap();
        assert!(status.success());

        // create trunk branch
        let status = Command::new("git").args(&["checkout","-b","trunk"]).status().unwrap();
        assert!(status.success());

        // empty commit to actually create trunk branch
        let status = Command::new("git")
            .args(&["commit","--allow-empty","-m","hello"]).status().unwrap();
        assert!(status.success());

        // create a fake branch to test deletion
        let status = Command::new("git").args(&["branch","hotfix"]).status().unwrap();
        assert!(status.success());

        Self{ working_dir }
    }

    fn path(&self) -> &Path {
        self.working_dir.path()
    }
}

lazy_static! {
    static ref TEST_STATE: TestState = TestState::new("git_pr_test");
}

#[test]
fn version() {
    println!("TempDir='{:?}'", TEST_STATE.path());

    let git = Git::new();
    let version = git.version().unwrap();
    assert!(version.starts_with("git version 2"));
}

#[test]
fn fetch_and_prune() {
    println!("TempDir='{:?}'", TEST_STATE.path());

    let git = Git::new();
    git.fetch_prune().unwrap();
}

#[test]
fn can_list_all_branches() {
    println!("TempDir='{:?}'", TEST_STATE.path());

    let git = Git::new();
    let branches = git.all_branches().unwrap();
    assert!(branches.contains("trunk"));
}

// Cleaning PRs requires that we identify "old" branches (those which have been merged into trunk),
// and that we delete those branches. Because the tests run in parallel, we need to ensure that our
// check for the existence of the "hotfix" branch always happens *before* our attempt to delete the
// "hotfix" branch. So this test case exercises all the git client functionality we would need in
// order to implement the "pr-clean" subcommand.
#[test]
fn could_clean() {
    println!("TempDir='{:?}'", TEST_STATE.path());

    let git = Git::new();
    let branches = git.merged_branches().unwrap();
    assert!(branches.contains("hotfix"));

    git.delete_branch("hotfix").unwrap();
    let branches = git.all_branches().unwrap();
    assert!(!branches.contains("hotfix"));
}

#[test]
fn can_get_hash_of_head() {
    println!("TempDir='{:?}'", TEST_STATE.path());

    // The hash will change every time, but this is one of the few git commands for which we can
    // know the exact length of the output. Weak, but best we can do until we add more capabilities
    // to the client.
    let git = Git::new();
    let hash = git.rev_parse_head().unwrap();
    assert_eq!(hash.len(), 7);
}

#[test]
fn can_create_new_branch() {
    println!("TempDir='{:?}'", TEST_STATE.path());

    // Show that we can create a new branch in this repo, and verify its existence by querying the
    // list of branches and showing that this new branch is among them.
    let git = Git::new();
    git.create_branch("knurt").unwrap();
    let branches = git.all_branches().unwrap();
    assert!(branches.contains("knurt"));
}
