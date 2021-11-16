//! Pull request management for bare repos

mod branch_name;
mod local_branch;
mod output_list;

use local_branch::LocalBranch;
use output_list::OutputList;
use regex::Regex;
use std::io;
use std::path::Path;
use std::process::Command;
use std::process::ExitStatus;

pub type LocalBranches = OutputList<LocalBranch>;

/// Wrapper for the git command line program
///
/// If you think of git's command line interface as a sortof API, then this type is our API client.
/// It provides only those features that we need from git in order to set up our PR workflow. It is
/// intentionally bare-bones: for testing purposes, we want to do as much logic as possible without
/// relying on an external tool or service. 
pub struct Git {
    // The path to the version of git we'd like to use. Nominally, this would always be "git", but
    // we allow it to be specified in tests (see the unit tests for this module) so that we can
    // test some functionality against mock implementations of git. This makes it easier to
    // exercise edge cases without having to make real git jump through hoops.
    pub program: String,

    // Path to the repository. This is `.` by default in production, but for tests we want to be
    // able to invoke git as though we were in a temporary, test-specific directory.
    pub working_dir: Box<dyn AsRef<Path>>,
}


/// Custom Error Type for Git Problems
///
/// Lower-level errors are wrapped into this type so that we can return a uniform error type
/// without losing any of the original context.
#[derive(Debug)]
pub enum GitError {

    /// We encountered an error while launching or waiting on the child process.
    Io(io::Error),

    /// The child process ran, but returned a non-zero exit code.
    Exit(ExitStatus)
}

impl From<io::Error> for GitError {
    /// Wrap an [`io::Error`] in a [`GitError::Io`]
    fn from(other: io::Error) -> GitError {
        GitError::Io(other)
    }
}

fn assert_success(status: ExitStatus) -> Result<(),GitError> {
    match status.success() {
        true => Ok(()),
        false => Err(GitError::Exit(status))
    }
}

impl Git {
    /// Create a new "git client".
    ///
    /// This will rely on the operating system to infer the appropriate path to git, based on the
    /// current environment (just like your shell does it).
    pub fn new() -> Git {
        Git{ program: String::from("git"), working_dir: Box::new(String::from(".")) }
    }

    /// Report the version of the underlying git binary.
    ///
    /// This is equivalent to invoking `git --version` on the command line. Making this transparent
    /// to users of `git-pr` may help them begin to debug unexpected issues; For example, `git-pr`
    /// may not work correctly with very old versions of git.
    pub fn version(&self) -> Result<String,GitError> {
        let output = Command::new(&self.program)
            .arg("-C").arg(self.working_dir.as_ref().as_ref())
            .arg("--version").output()?;
        assert_success(output.status)?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Update the local branch list.
    ///
    /// This asks git to download the current list of branches from the remote server, cleaning up
    /// local references to any that have been deleted. This ensures that the user is able to see
    /// the same set of "current PRs" as their collaborators.
    pub fn fetch_prune(&self) -> Result<(),GitError> {
        let status = Command::new(&self.program)
            .arg("-C").arg(self.working_dir.as_ref().as_ref())
            .args(&["fetch","--prune"]).status()?;
        assert_success(status)?;

        Ok(())
    }

    /// Produce a list of branch names.
    ///
    /// This asks the configured `git` binary to produce a list of *all* known branches, including
    /// references to remote branches. It is from this list that we can produce the list of
    /// "current PRs".
    pub fn all_branches(&self) -> Result<String,GitError> {
        let output = Command::new(&self.program)
            .arg("-C").arg(self.working_dir.as_ref().as_ref())
            .args(&["branch","-a"]).output()?;
        assert_success(output.status)?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Produce a list of PRs which are elligible for deletion.
    pub fn merged_branches(&self) -> Result<LocalBranches,GitError> {
        let output = Command::new(&self.program)
            .arg("-C").arg(self.working_dir.as_ref().as_ref())
            .args(&["branch","--merged","trunk"]).output()?;
        assert_success(output.status)?;

        Ok(String::from_utf8_lossy(&output.stdout).parse::<LocalBranches>().unwrap())
    }

    /// Get the hash of the HEAD commit.
    ///
    /// This is useful for creating new PR branches, since we can use this value as a way to
    /// indicate the "base" of the current work. This function takes advantage of the `core.abbrev`
    /// config value, and will return a hash of the indicated length. If this value is not
    /// specificed, git will return the shortest hash necessary to uniquely identify the commit.
    pub fn rev_parse_head(&self) -> Result<String,GitError> {
        let output = Command::new(&self.program)
            .arg("-C").arg(self.working_dir.as_ref().as_ref())
            .args(&["rev-parse","--short","HEAD"]).output()?;
        assert_success(output.status)?;

        Ok(String::from_utf8_lossy(&output.stdout).trim_end().to_string())
    }

    /// Create a new branch
    ///
    /// Used with [`rev_parse_head`] as part of the `git-pr-create` tool. Pull requests are
    /// expressed as branches with a certain naming pattern (`pr-name/hash`). So in our system,
    /// creating a branch and creating a pull request are the same operation!
    pub fn create_branch(&self, name: &str) -> Result<(), GitError> {
        let status = Command::new(&self.program)
            .arg("-C").arg(self.working_dir.as_ref().as_ref())
            .args(&["checkout","-b",name]).status()?;
        assert_success(status)?;

        Ok(())
    }

    /// Delete a branch
    ///
    /// Won't delete unmerged branches.
    pub fn delete_branch(&self, name: &str) -> Result<(), GitError> {
        let status = Command::new(&self.program)
            .arg("-C").arg(self.working_dir.as_ref().as_ref())
            .args(&["branch","-d",name]).status()?;
        assert_success(status)?;

        Ok(())
    }

    /// Push a branch to `origin` and set upstream tracking
    ///
    /// Used in `git-pr-create` to notify other developers that a new PR has been created.
    pub fn push_upstream(&self, name: &str) -> Result<(), GitError> {
        let status = Command::new(&self.program)
            .arg("-C").arg(self.working_dir.as_ref().as_ref())
            .args(&["push","-u","origin",name]).status()?;
        assert_success(status)?;

        Ok(())
    }
}


/// Search a string for names matching our PR Pattern.
///
/// Given a string like the following (ostensibly the output of `git branch -a`):
///
/// ```console
///   cool-branch
/// * trunk
///   remotes/origin/new-idea/5
///   remotes/origin/hotfix/0
/// ```
/// 
/// this function will return a vector of two strings: "new-idea" and "hotfix". That's because our
/// criteria for pull request names is:
///
/// * must begin with "remotes/origin/"
/// * must end with one or more hex digits
pub fn extract_pr_names(branches: &str) -> Vec<String> {

    // It's okay to call `.unwrap()` here, because we know that the regexes compile as long as the
    // "parse_branches_into_pr_list" unit test passes.
    let begins_with_remote_ref: Regex = Regex::new(r"^ *\** remotes/origin/").unwrap();
    let ends_with_hex: Regex = Regex::new(r"/[a-f\d]+$").unwrap();

    // Select any branches which match *both* of the regexes defined above.
    let pr_branches: Vec<&str> = branches.lines()
        .filter(|b| begins_with_remote_ref.is_match(b))
        .filter(|b| ends_with_hex.is_match(b))
        .collect();

    // Transform each branch "remotes/origin/blah/N" into a PR Name: "blah".  This has some
    // ownership repercussions that I don't quite understand, but they are outlined in
    // https://github.com/robertdfrench/git-pr/issues/7 .
    let mut pr_names = vec![];
    for branch in pr_branches {
        let branch = begins_with_remote_ref.replace_all(&branch, "");
        let branch = ends_with_hex.replace_all(&branch, "");
        pr_names.push(branch.to_string())
    }

    pr_names
}

pub fn extract_deletable_branches(branches: &str) -> Vec<String> {
    branches.lines()
        .filter(|b| !b.starts_with("*")) // skip the current branch
        .map(|b| b.trim_start()) // remove left-hand gutter characters
        .map(|b| b.trim_end()) // remove newlines
        .filter(|b| *b != "trunk")
        .map(|b| b.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Implementing this above produces a warning, since the function is (by design) never used by
    // other application code. Since it is only used in this module, we implement this function
    // local to this module, thus eliminating the dead code warning.
    impl Git {
        fn with_path(path: String) -> Git {
            let working_dir = Box::new(".");

            Git{ program: path, working_dir }
        }
    }

    macro_rules! crate_target {
        ($name:expr) => {
            match cfg!(debug_assertions) {
                true => format!("./target/debug/{}", $name),
                false => format!("./target/release/{}", $name),
            }
        };
    }

    // Verify that we out Git "client" can query the underlying git for its version info. The
    // `fake_git` program (defined in src/bin/fake_git.rs) will respond with a known string if
    // invoked with the "--version" argument.
    #[test]
    fn query_version_info() {
        let fake_git = Git::with_path(crate_target!("fake_git"));
        let version = fake_git.version().unwrap();
        assert!(version.starts_with("fake_git version 1"));
    }

    // Test how we handle failure when invoking git.
    //
    // In any reasonable scenario, `git --version` will not fail. We check this path to validate
    // the error handling pattern used in `Git::version` so that we might use this pattern
    // elsewhere.
    #[test]
    #[should_panic]
    fn query_version_failure() {
        let failing_git = Git::with_path(crate_target!("failing_git"));
        failing_git.version().unwrap();
    }

    // Show that we can extract a list of pr names from the output of `git branch -a`.
    #[test]
    fn parse_branches_into_pr_list() {
        let branches: &'static str = "
          local-junk
        * stuff/I/wrote
          trunk
          remotes/origin/first-pr/000000
          remotes/origin/second/f3f3f3
          remotes/origin/not-being-tracked
          remotes/origin/has-a-directory-but/still-not-being-tracked
        ";

        let pr_names = extract_pr_names(branches);
        assert_eq!(pr_names.len(), 2);
        assert_eq!(pr_names[0], "first-pr");
        assert_eq!(pr_names[1], "second");
    }

    #[test]
    fn can_detect_merged_branches() {
        let fake_git = Git::with_path(crate_target!("fake_git"));
        let mut merged_branches = fake_git.merged_branches().unwrap();
        assert!(merged_branches.any(|lb| lb.name.value == "already-been-merged"));
    }

    #[test]
    fn can_issue_delete_statement() {
        let fake_git = Git::with_path(crate_target!("fake_git"));
        fake_git.delete_branch("already-been-merged").unwrap();
    }

    #[test]
    fn identify_branches_for_deletion() {
        let merged_branches = vec![
            "  one",
            "* two",
            "  trunk",
            "  three",
            ""
        ].join("\n");

        let pr_names = extract_deletable_branches(&merged_branches);
        assert_eq!(pr_names.len(), 2);
        assert_eq!(pr_names[0], "one");
        assert_eq!(pr_names[1], "three");
    }

    // fake_git returns a constant, known hash, so we check for that.
    #[test]
    fn get_hash_of_current_commit() {
        let fake_git = Git::with_path(crate_target!("fake_git"));
        let hash = fake_git.rev_parse_head().unwrap();
        assert_eq!(hash, "1234567");
    }

    // We call `create_branch` to ensure it doesn't throw an error, but we don't have enough
    // tooling in `fake_git` to warrant checking for a change in state afterwards -- this is more
    // appropriate for an integration test with real git.
    #[test]
    fn create_new_branch() {
        let fake_git = Git::with_path(crate_target!("fake_git"));
        fake_git.create_branch("hotfix").unwrap();
    }
}
