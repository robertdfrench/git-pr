//! Pull request management for bare repos


use lazy_static::lazy_static; // Suggested by regex crate docs. We use this to compile regexes at
                              // source code compile-time, saving crucial picoseconds at runtime.
use regex::Regex;
use std::io;
use std::process::Command;
use std::process::ExitStatus;


/// Wrapper for the git command line program
///
/// If you think of git's command line interface as a sortof API, then this type is our API client.
/// It provides only those features that we need from git in order to set up our PR workflow. It is
/// intentionally bare-bones: for testing purposes, we want to do as much logic as possible without
/// relying on an external tool or service. 
#[derive(Debug)]
pub struct Git {
    // The path to the version of git we'd like to use. Nominally, this would always be "git", but
    // we allow it to be specified in tests (see the unit tests for this module) so that we can
    // test some functionality against mock implementations of git. This makes it easier to
    // exercise edge cases without having to make real git jump through hoops.
    program: String,
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
        Git{ program: String::from("git") }
    }

    /// Report the version of the underlying git binary.
    ///
    /// This is equivalent to invoking `git --version` on the command line. Making this transparent
    /// to users of `git-pr` may help them begin to debug unexpected issues; For example, `git-pr`
    /// may not work correctly with very old versions of git.
    pub fn version(&self) -> Result<String,GitError> {
        let output = Command::new(&self.program).arg("--version").output()?;
        assert_success(output.status)?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Update the local branch list.
    ///
    /// This asks git to download the current list of branches from the remote server, cleaning up
    /// local references to any that have been deleted. This ensures that the user is able to see
    /// the same set of "current PRs" as their collaborators.
    pub fn fetch_prune(&self) -> Result<(),GitError> {
        let status = Command::new(&self.program).args(&["fetch","--prune"]).status()?;
        assert_success(status)?;

        Ok(())
    }

    /// Produce a list of branch names.
    ///
    /// This asks the configured `git` binary to produce a list of *all* known branches, including
    /// references to remote branches. It is from this list that we can produce the list of
    /// "current PRs".
    pub fn all_branches(&self) -> Result<String,GitError> {
        let output = Command::new(&self.program).args(&["branch","-a"]).output()?;
        assert_success(output.status)?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
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
/// * must end with one or more digits
pub fn extract_pr_names(branches: &str) -> Vec<String> {

    // Compile regexes at compile time, rather than compiling them at runtime every time this
    // function is invoked. Honestly, this might be overkill.
    lazy_static! {
        static ref BEGINS_WITH_REMOTE_REF: Regex = Regex::new(r"^ *\** remotes/[^/]+/").unwrap();
        static ref ENDS_WITH_DIGIT: Regex = Regex::new(r"/\d+$").unwrap();
    }

    // Select any branches which match *both* of the regexes defined above.
    let pr_branches: Vec<&str> = branches.lines()
        .filter(|b| BEGINS_WITH_REMOTE_REF.is_match(b))
        .filter(|b| ENDS_WITH_DIGIT.is_match(b))
        .collect();

    // Transform each branch "remotes/origin/blah/N" into a PR Name: "blah".  This has some
    // ownership repercussions that I don't quite understand, but they are outlined in
    // https://github.com/robertdfrench/git-pr/issues/7 .
    let mut pr_names = vec![];
    for branch in pr_branches {
        let branch = BEGINS_WITH_REMOTE_REF.replace_all(&branch, "");
        let branch = ENDS_WITH_DIGIT.replace_all(&branch, "");
        pr_names.push(branch.to_string())
    }

    pr_names
}


#[cfg(test)]
mod tests {
    use super::*;

    // Implementing this above produces a warning, since the function is (by design) never used by
    // other application code. Since it is only used in this module, we implement this function
    // local to this module, thus eliminating the dead code warning.
    impl Git {
        fn with_path(path: String) -> Git {
            Git{ program: path }
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
          remotes/origin/first-pr/0
          remotes/origin/second/3
          remotes/origin/not-being-tracked
          remotes/origin/has-a-directory-but/still-not-being-tracked
        ";

        let pr_names = extract_pr_names(branches);
        assert_eq!(pr_names.len(), 2);
        assert_eq!(pr_names[0], "first-pr");
        assert_eq!(pr_names[1], "second");
    }

    // Show that we can extract a list of pr names even when the remote is not "origin"
    #[test]
    fn parse_branches_from_custom_remotes() {
        let branches: &'static str = "
          remotes/yabba-dabba-doo/first-pr/0
          remotes/yabba{dabba}doo/second/0
          remotes/yabba/dabba/doo/third/0
          remotes/yabba dabba doo/fourth/0
        ";

        let pr_names = extract_pr_names(branches);
        assert_eq!(pr_names.len(), 4);
        assert_eq!(pr_names[0], "first-pr");
        assert_eq!(pr_names[1], "second");
        assert_eq!(pr_names[2], "dabba/doo/third");
        assert_eq!(pr_names[3], "fourth");
    }
}
