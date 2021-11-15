//! Parse the names of local branches

use crate::branch_name::BranchName;
use std::str::FromStr;

#[derive(Debug)]
pub enum ParseError {
    MissingName
}

#[derive(Debug)]
pub struct LocalBranch {
    pub name: BranchName,
    pub is_head: bool
}

impl LocalBranch {
    pub fn new(is_head: bool, name: &str) -> Self {
        // It's always safe to unwrap this since BranchName::parse is [`Infallible`].
        let name = name.parse::<BranchName>().unwrap();
        Self{ is_head, name }
    }
    pub fn looks_like_pr(&self) -> bool {
        self.name.looks_like_pr()
    }
}

impl FromStr for LocalBranch {
    type Err = ParseError;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let mut components = line.split_whitespace();

        // The line will either look like "  branch/name" or "* branch/name". If it doesn't, then
        // something is wrong with git.
        match components.next() {
            None => Err(ParseError::MissingName),
            Some("*") => match components.next() {
                None => Err(ParseError::MissingName),
                Some(name) => Ok(LocalBranch::new(true, name))
            },
            Some(name) => Ok(LocalBranch::new(false, name))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse() {
        let _: LocalBranch = "trunk".parse().unwrap();
    }

    #[test]
    fn can_detect_head() {
        let trunk: LocalBranch = "* trunk".parse().unwrap();
        assert!(trunk.is_head);
    }

    #[test]
    fn unstarred_branches_are_not_head() {
        let other: LocalBranch = "  other/branch/123abc".parse().unwrap();
        assert!(!other.is_head);
    }
}
