use bevy::prelude::*;

/// A component that stores a character position in a text
/// Separated from `usize` to make it clear that it is a character position, not a byte position.
/// And prevents accidental usage as a byte position in `string[..byte_position]`
#[derive(Reflect, Default, Clone, Copy, Debug)]
pub struct CharPosition(pub usize);

impl std::ops::Add<CharPosition> for CharPosition {
    type Output = Self;

    fn add(self, rhs: CharPosition) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Sub<CharPosition> for CharPosition {
    type Output = Self;

    fn sub(self, rhs: CharPosition) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl std::ops::Add<usize> for CharPosition {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl std::ops::Sub<usize> for CharPosition {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl PartialOrd for CharPosition {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl PartialEq for CharPosition {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<usize> for CharPosition {
    fn eq(&self, other: &usize) -> bool {
        self.0 == *other
    }
}

impl PartialOrd<usize> for CharPosition {
    fn partial_cmp(&self, other: &usize) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialEq<CharPosition> for usize {
    fn eq(&self, other: &CharPosition) -> bool {
        *self == other.0
    }
}

impl PartialOrd<CharPosition> for usize {
    fn partial_cmp(&self, other: &CharPosition) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other.0)
    }
}

impl std::fmt::Display for CharPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<usize> for CharPosition {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

/// Get the byte position of a character placed at a given position in a string
pub fn get_byte_position(text: &str, char_position: CharPosition) -> usize {
    if char_position.0 < text.chars().count() {
        text.char_indices().nth(char_position.0).unwrap().0
    } else {
        text.len()
    }
}
