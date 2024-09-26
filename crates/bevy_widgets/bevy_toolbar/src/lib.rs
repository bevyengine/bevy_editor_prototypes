//! A toolbar widget for Bevy applications.
//!
//! Toolbars are a common UI element in many applications, providing quick access to frequently used commands,
//! and typically display small icons with on-hover tooltips.

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
