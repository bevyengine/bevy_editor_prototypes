//! Icons

/// Icons used in the editor UI. These icons map to specific
/// characters in  the Lucide icon font.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorIcon {
    /// A downward-facing chevron, often used to indicate
    /// collapsible sections or dropdowns.
    ChevronDown,
    /// A vertical grip icon, typically used for drag handles
    /// or reordering list items.
    GripVertical,
}

impl From<EditorIcon> for &'static str {
    fn from(value: EditorIcon) -> Self {
        match value {
            EditorIcon::ChevronDown => "",
            EditorIcon::GripVertical => "",
        }
    }
}

impl From<EditorIcon> for String {
    fn from(value: EditorIcon) -> Self {
        let s: &'static str = value.into();
        s.to_string()
    }
}
