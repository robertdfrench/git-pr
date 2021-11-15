//! Deal with branches by their filesystem-like names.
//!
//! The only thing this module does is to check whether a branch name looks like it belongs to a
//! PR. That is the only thing that local branches and remote branches have in common.
use std::str::FromStr;
use regex::Regex;


/// A filesystem-like name for a branch
#[derive(Debug)]
pub struct BranchName {
    pub value: String
}


impl BranchName {
    /// Does the branch name match our `pr/naming/schema/123abc`?
    pub fn looks_like_pr(&self) -> bool {
        let ends_with_hex: Regex = Regex::new(r"/[a-f\d]+$").unwrap();
        ends_with_hex.is_match(&self.value)
    }
}

impl FromStr for BranchName {

    // We are not using any methods that can fail, so we don't need an error type.
    type Err = std::convert::Infallible;

    /// Convert a `&str` into a BranchName
    ///
    /// This will allow us to manipulate the branch name with the methods from `impl BranchName`
    /// rather than (potentially clunkier) string-manipulation methods.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(BranchName{ value: String::from(s) })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse() {
        let _: BranchName = "trunk".parse().unwrap();
    }

    #[test]
    fn trunk_is_not_a_pr() {
        let trunk = "trunk".parse::<BranchName>().unwrap();
        assert!(!trunk.looks_like_pr());
    }

    #[test]
    fn can_identify_a_pr() {
        let pr_branch = BranchName::from_str("pr-name/abc123").unwrap();
        assert!(pr_branch.looks_like_pr());
    }
}
