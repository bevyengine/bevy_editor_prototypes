//! Simple non-destructive .bsn-hot-reloading utilizing retained scenes.
//!
//! Run with `--features="bevy/file_watcher"` to enable hot-reloading.
use bevy::prelude::*;
use bevy_bsn::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BsnPlugin)
        .add_systems(
            Startup,
            |mut commands: Commands, assets: Res<AssetServer>| {
                commands.spawn(Camera2d);

                let bsn = assets.load::<Bsn>("hello.bsn");
                commands.spawn(SceneRoot(bsn));
            },
        )
        .add_systems(Update, spawn_and_reload_scene)
        .run();
}

#[derive(Component)]
struct SceneRoot(Handle<Bsn>);

fn spawn_and_reload_scene(
    scene_root: Single<Entity, With<SceneRoot>>,
    mut events: EventReader<AssetEvent<Bsn>>,
    bsn_assets: Res<Assets<Bsn>>,
    app_registry: Res<AppTypeRegistry>,
    mut commands: Commands,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                let bsn = bsn_assets.get(*id).unwrap();

                let registry = app_registry.read();
                let dynamic_scene = BsnReflector::new(bsn, &registry)
                    .reflect_dynamic_scene()
                    .unwrap();

                commands.entity(*scene_root).retain_scene(dynamic_scene);
            }
            _ => {}
        }
    }
}
