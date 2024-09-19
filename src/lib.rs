use bevy::prelude::*;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, _app: &mut App) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_add_plugin() {
        App::new().add_plugins((MinimalPlugins, EditorPlugin));
    }
}
