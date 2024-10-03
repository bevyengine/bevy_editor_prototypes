//! An interactive, reflection-based inspector for Bevy ECS data in running applications.
//!
//! Data can be viewed and modified in real-time, with changes being reflected in the application.

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
