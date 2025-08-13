//! Color constants for the Bevy Editor
//! 
//! These colors are derived from the Figma design specifications and provide
//! a consistent dark theme across the editor interface.
//!
//! ## Usage
//!
//! Import and use color constants directly:
//!
//! ```rust
//! use bevy::prelude::*;
//! use bevy_editor_styles::colors::EditorColors;
//!
//! fn setup_ui(mut commands: Commands) {
//!     commands.spawn((
//!         Node::default(),
//!         BackgroundColor(EditorColors::BACKGROUND),
//!     ));
//! }
//! ```

use bevy::prelude::Color;

/// Central color constants for the Bevy Editor
/// 
/// All colors use sRGB values converted from the original CSS hex values
/// from the Figma design specification.
pub struct EditorColors;

impl EditorColors {
    // === Core Background Colors ===
    
    /// Main editor background color - CSS: #1F1F24
    pub const BACKGROUND: Color = Color::srgb(0.122, 0.122, 0.141);
    
    /// Panel and content area background - CSS: #2A2A2E  
    pub const PANEL_BACKGROUND: Color = Color::srgb(0.165, 0.165, 0.180);
    
    // === Button Colors ===
    
    /// Default button background - CSS: #36373B
    pub const BUTTON_DEFAULT: Color = Color::srgb(0.212, 0.216, 0.231);
    
    /// Button hover state - slightly lighter than default
    pub const BUTTON_HOVER: Color = Color::srgb(0.255, 0.259, 0.278);
    
    // === Text Colors ===
    
    /// Primary text color - CSS: #ECECEC
    pub const TEXT_PRIMARY: Color = Color::srgb(0.925, 0.925, 0.925);
    
    /// Muted/secondary text color - CSS: #838385
    pub const TEXT_MUTED: Color = Color::srgb(0.514, 0.514, 0.522);
    
    // === Accent Colors ===
    
    /// Primary accent blue for active states and highlights - CSS: #206EC9
    pub const ACCENT_BLUE: Color = Color::srgb(0.125, 0.431, 0.788);
    
    /// Brighter blue for hover states on active elements
    pub const ACCENT_BLUE_BRIGHT: Color = Color::srgb(0.145, 0.471, 0.828);
    
    // === Transform Gizmo Axis Colors ===
    
    /// X-axis color (red) - #AB4051
    pub const AXIS_X: Color = Color::srgb(0.671, 0.251, 0.318);
    
    /// Y-axis color (green) - #5D8D0A
    pub const AXIS_Y: Color = Color::srgb(0.365, 0.553, 0.039);
    
    /// Z-axis color (blue) - #2160A3
    pub const AXIS_Z: Color = Color::srgb(0.129, 0.376, 0.639);
    
    // === Grid and Border Colors ===
    
    /// Major grid lines - CSS: #414142
    pub const GRID_MAJOR: Color = Color::srgb(0.255, 0.255, 0.259);
    
    /// Minor grid lines and borders - CSS: #303030
    pub const GRID_MINOR: Color = Color::srgb(0.188, 0.188, 0.188);
    
    /// General border color
    pub const BORDER: Color = Color::srgb(0.180, 0.180, 0.184);
    
    // === Status Colors ===
    
    /// Success/enabled state - green
    pub const SUCCESS: Color = Color::srgb(0.36, 0.7, 0.05);
    
    /// Error/disabled state - red  
    pub const ERROR: Color = Color::srgb(0.8, 0.3, 0.3);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn color_constants_are_valid() {
        // Ensure all colors are in valid sRGB range [0.0, 1.0]
        let colors = [
            EditorColors::BACKGROUND,
            EditorColors::PANEL_BACKGROUND,
            EditorColors::BUTTON_DEFAULT,
            EditorColors::TEXT_PRIMARY,
            EditorColors::ACCENT_BLUE,
        ];
        
        for color in colors {
            let [r, g, b, a] = color.to_srgba().to_array();
            assert!((0.0..=1.0).contains(&r), "Red component out of range: {}", r);
            assert!((0.0..=1.0).contains(&g), "Green component out of range: {}", g);
            assert!((0.0..=1.0).contains(&b), "Blue component out of range: {}", b);
            assert!((0.0..=1.0).contains(&a), "Alpha component out of range: {}", a);
        }
    }
}