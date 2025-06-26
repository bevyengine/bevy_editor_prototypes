//! A Bevy plugin for displaying and editing transform data in a UI widget.
//! This plugin provides a UI template that displays the translation, rotation, and scale of a transform in a structured format.
//! It uses the `bevy_i_cant_believe_its_not_bsn` crate to create the UI template.
//!// The `TransformData` struct holds the translation, rotation, and scale data.
//! The `transform_widget_ui` function generates a UI template that displays this data in a user-friendly format.
//!// The UI is structured with labels and formatted values for each component of the transform.
//!// The UI is designed to be flexible and can be easily integrated into a Bevy application.
//!
use bevy::prelude::*;
use bevy_i_cant_believe_its_not_bsn::{Template, template};

/// The `TransformWidget` struct is a UI widget that displays transform data in a structured format.
pub struct TransformWidget;

impl TransformWidget {
    /// The `transform_widget_ui` function generates a UI template that displays the transform data in a structured/user-friendly format.
    pub fn draw_widget(transform_data: &Transform) -> Template {
        template! {
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                column_gap: Val::Px(4.0),
                ..Default::default()
            } => [
                (Text("Translation:".to_string()), TextFont::from_font_size(14.0));
                (Text(format!("x: {:.2}", transform_data.translation[0])), TextFont::from_font_size(14.0));
                (Text(format!("y: {:.2}", transform_data.translation[1])), TextFont::from_font_size(14.0));
                (Text(format!("z: {:.2}", transform_data.translation[2])), TextFont::from_font_size(14.0));
                (Text("Rotation:".to_string()), TextFont::from_font_size(14.0));
                (Text(format!("x: {:.2}", transform_data.rotation.to_array()[0])), TextFont::from_font_size(14.0));
                (Text(format!("y: {:.2}", transform_data.rotation.to_array()[1])), TextFont::from_font_size(14.0));
                (Text(format!("z: {:.2}", transform_data.rotation.to_array()[2])), TextFont::from_font_size(14.0));
                (Text(format!("w: {:.2}", transform_data.rotation.to_array()[3])), TextFont::from_font_size(14.0));
                (Text("Scale:".to_string()), TextFont::from_font_size(14.0));
                (Text(format!("x: {:.2}", transform_data.scale[0])), TextFont::from_font_size(14.0));
                (Text(format!("y: {:.2}", transform_data.scale[1])), TextFont::from_font_size(14.0));
                (Text(format!("z: {:.2}", transform_data.scale[2])), TextFont::from_font_size(14.0));
            ];
        }
    }

    /// The `draw_empty_selection` function generates a UI template that displays a message when no transform data is available for the selected entity.
    pub fn draw_missing_transform() -> Template {
        template! {
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                ..Default::default()
            } => [
                (Text("No transform data available for this entity.".into()), TextFont::from_font_size(14.0));
            ];
        }
    }

    /// The `draw_empty_selection` function generates a UI template that displays a message when no entity is selected.
    pub fn draw_empty_selection() -> Template {
        template! {
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                ..Default::default()
            } => [
                (Text("Select an entity to inspect".into()), TextFont::from_font_size(14.0));
            ];
        }
    }
}
