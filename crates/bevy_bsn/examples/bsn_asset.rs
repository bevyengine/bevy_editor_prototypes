//! Showcases loading and displaying a BSN asset.
use bevy::prelude::*;
use bevy_proto_bsn::*;

#[derive(Component)]
#[allow(dead_code)]
struct KeepHandle(Handle<Bsn>);

fn main() {
    App::new()
        .register_type::<RotateMe>()
        .add_plugins(DefaultPlugins)
        .add_plugins(BsnPlugin)
        .add_systems(
            Startup,
            |mut commands: Commands, assets: Res<AssetServer>| {
                commands.spawn((
                    Camera3d::default(),
                    Transform::from_xyz(0.7, 0.7, 1.0)
                        .looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
                ));

                let bsn = assets.load::<Bsn>("3d_scene.proto_bsn");
                commands.spawn(KeepHandle(bsn));
            },
        )
        .add_systems(Update, spawn_scene_on_load)
        .add_systems(Update, rotate)
        .run();
}

fn spawn_scene_on_load(
    mut events: EventReader<AssetEvent<Bsn>>,
    bsn_assets: Res<Assets<Bsn>>,
    app_registry: Res<AppTypeRegistry>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for event in events.read() {
        if let AssetEvent::Added { id } = event {
            let bsn = bsn_assets.get(*id).unwrap();
            info!("Loaded BSN: {:?}", bsn);

            let registry = app_registry.read();
            let dynamic_scene = BsnReflector::new(bsn, &registry)
                .with_asset_load(&asset_server)
                .reflect_dynamic_scene()
                .unwrap();

            commands.spawn_scene(dynamic_scene);
        };
    }
}

#[derive(Component, Default, Clone, Reflect)]
#[reflect(Component, Construct)]
struct RotateMe;

fn rotate(mut query: Query<&mut Transform, With<RotateMe>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() / 2.);
    }
}
