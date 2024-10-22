//! This example demonstrates how to use the `EntityInspector` plugin to inspect the transform of an entity.

use bevy::prelude::*;
use bevy_entity_inspector::{EntityInspector, EntityInspectorPlugin, InspectedEntity};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EntityInspectorPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::splat(5.0)).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)).looking_at(Vec3::ZERO, Vec3::Y),
        DirectionalLight::default(),
    ));

    commands.spawn((
        Transform::default(),
        Mesh3d(meshes.add(Cuboid::from_length(1.0))),
        MeshMaterial3d(materials.add(StandardMaterial::default())),
        InspectedEntity,
    ));

    commands.spawn((
        EntityInspector,
        Node {
            width: Val::Auto,
            border: UiRect::all(Val::Px(1.0)),
            overflow: Overflow::scroll_y(),
            ..default()
        },
        BorderRadius::all(Val::Px(5.0)),
        BorderColor(Color::srgb(0.5, 0.5, 0.5)),
        BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
    ));
}
