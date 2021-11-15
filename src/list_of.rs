//! A "List of <T>".
//!
//! Assuming T is `FromStr`, and you have lines of T source text, this will convert them into an
//! iterable container of T.
use std::str::FromStr;

pub struct ListOf<T: FromStr> {
    storage: Vec<T>
}

impl<T: FromStr> FromStr for ListOf<T> {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let storage = s.lines()
            .filter_map(|line| line.parse::<T>().ok())
            .rev().collect();

        Ok(Self{ storage })
    }
}

impl<T: FromStr> Iterator for ListOf<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.storage.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_list_of_numbers() {
        let source = "1\n2\nthree\n4";

        let list = source.parse::<ListOf<u8>>().unwrap();
        assert_eq!(list.storage.len(), 3);
    }

    #[test]
    fn can_iterate_list_of_numbers() {
        let source = "1\n2\nthree\n4";

        let list = source.parse::<ListOf<u8>>().unwrap();
        assert_eq!(list.collect::<Vec<u8>>(), vec![1,2,4]);
    }
}
