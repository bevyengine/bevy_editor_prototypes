use bevy::prelude::*;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, _app: &mut App) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::winit::WinitPlugin;

    #[test]
    fn can_add_plugin() {
        App::new()
            .add_plugins(DefaultPlugins.build().disable::<WinitPlugin>())
            .add_plugins(EditorPlugin);
    }
}
