//! A context menu abstraction for Bevy applications.
//!
//! This crate opens up a menu with a list of options when the user right-clicks (or otherwise triggers the context menu),
//! based on what the user has clicked on.

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
