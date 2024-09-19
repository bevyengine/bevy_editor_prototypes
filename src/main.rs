use bevy::prelude::*;
use bevy_editor_prototypes::EditorPlugin;

fn main() -> AppExit {
    App::new().add_plugins((DefaultPlugins, EditorPlugin)).run()
}
