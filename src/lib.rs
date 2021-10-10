//! Pull request management for bare repos
use std::io;
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug)]
pub struct Git {
    program: OsString,
}

impl Git {
    pub fn new() -> Git {
        Git{ program: OsString::from("git") }
    }
    pub fn with_path(path: &Path) -> Git {
        Git{ program: path.as_os_str().to_os_string() }
    }
    pub fn version(&self) -> io::Result<String> {
        let output = Command::new(&self.program).arg("--version").output()?;
        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "git --version"));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    pub fn fetch_prune(&self) -> io::Result<()> {
        let status = Command::new(&self.program).args(&["fetch","--prune"]).status()?;
        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "git fetch --prune"));
        }
        Ok(())
    }
    pub fn all_branches(&self) -> io::Result<String> {
        let output = Command::new(&self.program).args(&["branch","-a"]).output()?;
        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "git branch -a"));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

pub fn extract_pr_names(branches: &str) -> Vec<String> {
    lazy_static! {
        static ref BEGINS_WITH_REMOTE_REF: Regex = Regex::new(r"^ *\** remotes/origin/").unwrap();
        static ref ENDS_WITH_DIGIT: Regex = Regex::new(r"/\d+$").unwrap();
    }
    let pr_branches: Vec<&str> = branches.lines()
        .filter(|b| BEGINS_WITH_REMOTE_REF.is_match(b))
        .filter(|b| ENDS_WITH_DIGIT.is_match(b))
        .collect();

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

    #[test]
    fn query_version_info() {
        let path = Path::new(".").join("fake_git.sh");
        let fake_git = Git::with_path(&path);
        let version = fake_git.version().unwrap();
        assert!(version.starts_with("fake_git version 1"));
    }

    #[test]
    #[should_panic]
    fn query_version_failure() {
        let path = Path::new(".").join("failing_git.sh");
        let failing_git = Git::with_path(&path);
        failing_git.version().unwrap();
    }

    #[test]
    fn parse_branches_into_pr_list() {
        let branches: &'static str = "
          local-junk
        * stuff/I/wrote
          trunk
          remotes/origin/first-pr/0
          remotes/origin/second/3
        ";

        let pr_names = extract_pr_names(branches);
        assert_eq!(pr_names.len(), 2);
        assert_eq!(pr_names[0], "first-pr");
        assert_eq!(pr_names[1], "second");
    }
}
