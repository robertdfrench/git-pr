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
