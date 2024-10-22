//! This crate provides a set of widgets, which are used text input

pub mod drag_input;
pub mod input_field;
pub mod text_event_mirror;
pub mod validate_highlight;

use bevy::prelude::*;
use bevy_text_editing::*;

pub struct FieldFormsPlugin;

impl Plugin for FieldFormsPlugin {
    fn build(&self, app: &mut App) {

        if !app.is_plugin_added::<EditableTextLinePlugin>() {
            app.add_plugins(EditableTextLinePlugin);
        }

        app.add_plugins(input_field::InputFieldPlugin::<i8>::default());
        app.add_plugins(drag_input::DragInputPlugin::<i8>::default());

        app.add_plugins(input_field::InputFieldPlugin::<i16>::default());
        app.add_plugins(drag_input::DragInputPlugin::<i16>::default());

        app.add_plugins(input_field::InputFieldPlugin::<i32>::default());
        app.add_plugins(drag_input::DragInputPlugin::<i32>::default());

        app.add_plugins(input_field::InputFieldPlugin::<i64>::default());
        app.add_plugins(drag_input::DragInputPlugin::<i64>::default());

        app.add_plugins(input_field::InputFieldPlugin::<i128>::default());
        app.add_plugins(drag_input::DragInputPlugin::<i128>::default());

        app.add_plugins(input_field::InputFieldPlugin::<u8>::default());
        app.add_plugins(drag_input::DragInputPlugin::<u8>::default());

        app.add_plugins(input_field::InputFieldPlugin::<u16>::default());
        app.add_plugins(drag_input::DragInputPlugin::<u16>::default());

        app.add_plugins(input_field::InputFieldPlugin::<u32>::default());
        app.add_plugins(drag_input::DragInputPlugin::<u32>::default());

        app.add_plugins(input_field::InputFieldPlugin::<u64>::default());
        app.add_plugins(drag_input::DragInputPlugin::<u64>::default());

        app.add_plugins(input_field::InputFieldPlugin::<u128>::default());
        app.add_plugins(drag_input::DragInputPlugin::<u128>::default());

        app.add_plugins(input_field::InputFieldPlugin::<f32>::default());
        app.add_plugins(drag_input::DragInputPlugin::<f32>::default());

        app.add_plugins(input_field::InputFieldPlugin::<f64>::default());
        app.add_plugins(drag_input::DragInputPlugin::<f64>::default());

        app.add_plugins(input_field::InputFieldPlugin::<String>::default());

        app.add_plugins(text_event_mirror::TextEventMirrorPlugin);
        app.add_plugins(validate_highlight::SimpleBorderHighlightPlugin);
    }
}
