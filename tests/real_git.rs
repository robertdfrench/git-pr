//! Test the git "client" wrapper against the real git binary.
use libgitpr::Git;

#[test]
fn version() {
    let git = Git::new();
    let version = git.version().unwrap();
    assert!(version.starts_with("git version 2"));
}

#[test]
fn fetch_and_prune() {
    let git = Git::new();
    git.fetch_prune().unwrap();
}

#[test]
fn can_list_all_branches() {
    let git = Git::new();
    let branches = git.all_branches().unwrap();
    println!("{:?}", branches);
    assert!(branches.contains("trunk"));
}
