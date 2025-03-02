//! Showcases the usage of prefabs to conveniently load and retain BSN asset instances.
//!
//! Run with `--features="bevy/file_watcher"` to enable hot-reloading.
use bevy::prelude::*;
use bevy_proto_bsn::*;

fn main() {
    App::new()
        .register_type::<RotateMe>()
        .add_plugins(DefaultPlugins)
        .add_plugins(BsnPlugin)
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn((
                Camera3d::default(),
                Transform::from_xyz(0.0, 0.7, 1.0).looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
            ));

            commands.spawn_scene(pbsn! {
                Prefab(@"3d_scene.proto_bsn")
            });
        })
        .add_systems(Update, rotate)
        .run();
}

#[derive(Component, Default, Clone, Reflect)]
#[reflect(Component, Construct)]
struct RotateMe;

fn rotate(mut query: Query<&mut Transform, With<RotateMe>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() / 2.);
    }
}
