//! # Undo/Redo Example for Bevy
//!
//! This example demonstrates how to use the undo/redo functionality in a simple Bevy application.
//! It creates a cube that can be moved left and right, with the ability to undo and redo these movements.
//!
//! ## Features
//!
//! - A movable cube controlled by keyboard input
//! - Undo/redo functionality for cube movements
//! - Visual display of the undo history
//!
//! ## Controls
//!
//! - `A`: Move the cube left
//! - `D`: Move the cube right
//! - `Ctrl + Z`: Undo the last movement
//! - `Ctrl + Shift + Z`: Redo the last undone movement
//!
//! ## Code Overview
//!
//! The example consists of several key components:
//!
//! 1. `setup`: Initializes the scene with a cube, camera, and UI text.
//! 2. `move_cube`: Handles the cube movement based on keyboard input.
//! 3. `send_undo_event`: Listens for undo/redo key combinations and sends appropriate events.
//! 4. `write_undo_text`: Updates the UI text to display the current undo history.
//!
//! ## Important Notes
//!
//! - The `UndoMarker` component is added to the cube to enable undo/redo functionality for it.
//! - `OneFrameUndoIgnore` is used to prevent the initial Transform component addition from being recorded in the undo history.
//! - The `auto_reflected_undo::<Transform>()` call sets up automatic undo/redo tracking for the Transform component.
//!
//! ## Running the Example
//!
//! To run this example, ensure you have Bevy and the undo plugin added to your project's dependencies.
//! Then, you can run it using `cargo run --example undo_demo` (assuming you've named this file `examples/undo_demo.rs`).

use bevy::prelude::*;
use bevy_undo::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UndoPlugin)
        .auto_reflected_undo::<Transform>()
        .add_systems(Startup, setup)
        .add_systems(Update, (move_cube, send_undo_event, write_undo_text))
        .run();
}

#[derive(Component)]
struct Controller;

fn setup(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube_mesh = meshes.add(Cuboid::from_length(1.0));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.5, 0.3),
        ..Default::default()
    });

    cmd.spawn((Mesh3d(cube_mesh), MeshMaterial3d(material)))
        .insert(Controller)
        .insert(UndoMarker) //Only entities with this marker will be able to undo
        .insert(OneFrameUndoIgnore::default()); // To prevent adding "Transform add" change in change chain

    cmd.spawn((
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        Camera3d::default(),
    ));

    cmd.spawn((Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        justify_content: JustifyContent::Start,
        align_items: AlignItems::Start,
        ..Default::default()
    },))
        .with_children(|parent| {
            parent.spawn((Text::new(""), TextFont::default()));
        });
}

fn move_cube(
    inputs: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Controller>>,
    time: Res<Time>,
) {
    let speed = 10.0;
    if inputs.pressed(KeyCode::KeyA) {
        for mut transform in &mut query {
            transform.translation += Vec3::new(-1.0, 0.0, 0.0) * time.delta_secs() * speed;
        }
    }

    if inputs.pressed(KeyCode::KeyD) {
        for mut transform in &mut query {
            transform.translation += Vec3::new(1.0, 0.0, 0.0) * time.delta_secs() * speed;
        }
    }
}

fn send_undo_event(mut events: MessageWriter<UndoRedo>, inputs: Res<ButtonInput<KeyCode>>) {
    if inputs.just_pressed(KeyCode::KeyZ)
        && inputs.pressed(KeyCode::ControlLeft)
        && !inputs.pressed(KeyCode::ShiftLeft)
    {
        events.write(UndoRedo::Undo);
    }

    if inputs.just_pressed(KeyCode::KeyZ)
        && inputs.pressed(KeyCode::ControlLeft)
        && inputs.pressed(KeyCode::ShiftLeft)
    {
        events.write(UndoRedo::Redo);
    }
}

fn write_undo_text(
    mut query: Query<&mut Text>,
    change_chain: Res<ChangeChain>, //Change chain in UndoPlugin
) {
    for mut text in &mut query {
        let mut t = "Registered changes\n".to_string();

        for change in change_chain.changes.iter() {
            t = format!("{}{}\n", t, change.debug_text());
        }

        text.0 = t;
    }
}
