//! Showcases loading and displaying a BSN asset.
use bevy::prelude::*;
use bevy_proto_bsn::*;

#[derive(Component)]
#[allow(dead_code)]
struct KeepHandle(Handle<Bsn>);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BsnPlugin)
        .add_systems(
            Startup,
            |mut commands: Commands, assets: Res<AssetServer>| {
                commands.spawn(Camera2d);

                let bsn = assets.load::<Bsn>("hello.bsn");
                commands.spawn(KeepHandle(bsn));
            },
        )
        .add_systems(Update, spawn_scene_on_load)
        .run();
}

fn spawn_scene_on_load(
    mut events: EventReader<AssetEvent<Bsn>>,
    bsn_assets: Res<Assets<Bsn>>,
    app_registry: Res<AppTypeRegistry>,
    mut commands: Commands,
) {
    for event in events.read() {
        if let AssetEvent::Added { id } = event {
            let bsn = bsn_assets.get(*id).unwrap();
            info!("Loaded BSN: {:?}", bsn);

            let registry = app_registry.read();
            let dynamic_scene = BsnReflector::new(bsn, &registry)
                .reflect_dynamic_scene()
                .unwrap();

            commands.spawn_scene(dynamic_scene);
        };
    }
}
