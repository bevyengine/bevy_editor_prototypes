//! Test example for the property panel integration

use bevy::prelude::*;
use bevy_entity_inspector::{create_dark_inspector_theme, InspectorPlugin};

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Velocity {
    dx: f32,
    dy: f32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Health {
    current: u32,
    max: u32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InspectorPlugin)
        .insert_resource(create_dark_inspector_theme())
        .register_type::<Position>()
        .register_type::<Velocity>()
        .register_type::<Health>()
        .add_systems(Startup, setup_test_entities)
        .run();
}

fn setup_test_entities(mut commands: Commands) {
    // Create some test entities with components
    commands.spawn((
        Name::new("Player"),
        Position {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        Velocity { dx: 0.5, dy: -0.2 },
        Health {
            current: 80,
            max: 100,
        },
    ));

    commands.spawn((
        Name::new("Enemy"),
        Position {
            x: -5.0,
            y: 0.0,
            z: 2.0,
        },
        Velocity { dx: -1.0, dy: 0.0 },
        Health {
            current: 45,
            max: 60,
        },
    ));

    commands.spawn((
        Name::new("Static Object"),
        Position {
            x: 0.0,
            y: 10.0,
            z: 0.0,
        },
    ));
}
