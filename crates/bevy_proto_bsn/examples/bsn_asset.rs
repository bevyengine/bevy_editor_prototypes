//! This example shows how to load/reflect BSN scene and spawn it in the world.
//!
//! Also check out the `bsn_asset_prefab` example for a more convenient way to load and retain BSN assets.
//!
//! Run with `--features="bevy/file_watcher"` to enable hot-reloading.
use bevy::prelude::*;
use bevy_proto_bsn::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BsnPlugin)
        .add_systems(
            Startup,
            |mut commands: Commands, assets: Res<AssetServer>| {
                commands.spawn(Camera2d);

                let bsn = assets.load::<ReflectedBsn>("hello.proto_bsn");
                commands.spawn(SceneRoot(bsn));
            },
        )
        .add_systems(Update, spawn_and_reload_scene)
        .run();
}

#[derive(Component)]
#[allow(dead_code)]
struct SceneRoot(Handle<ReflectedBsn>);

fn spawn_and_reload_scene(
    scene_root: Single<Entity, With<SceneRoot>>,
    mut events: MessageReader<AssetEvent<ReflectedBsn>>,
    bsn_assets: Res<Assets<ReflectedBsn>>,
    mut commands: Commands,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                let bsn = bsn_assets.get(*id).unwrap();
                let dynamic_scene = bsn.clone().into_dynamic_scene();

                // Retain the scene on the root entity.
                // Retention stores some metadata on the entities to ensures that the scene is intelligently updated on asset hot reload.
                commands.entity(*scene_root).retain_scene(dynamic_scene);
                // For regular spawning without the retention overhead, use
                //  commands.entity(*scene_root).construct_scene(dynamic_scene);
                //   OR
                //  commands.spawn_scene(dynamic_scene);
            }
            _ => {}
        }
    }
}
