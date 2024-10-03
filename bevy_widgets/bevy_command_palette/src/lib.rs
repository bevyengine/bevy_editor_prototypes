//! A command palette for Bevy applications.
//!
//! This lists a number of commands that can be executed by the user,
//! allowing for quick access to a variety of functionality.
//!
//! Search and keyboard shortcuts will both be supported.

/// an add function that adds two numbers
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
