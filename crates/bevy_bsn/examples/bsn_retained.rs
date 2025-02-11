//! Super Sheep-Counter 2000
//!
//! An all-in-one numerical ruminant package.
//!
//! This example is originally from `i-cant-believe-its-not-bsn`.
use bevy::{color::palettes::css, prelude::*};

use bevy_bsn::{Scene, *};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BsnPlugin)
        .add_plugins(sheep_plugin)
        .run();
}

fn sheep_plugin(app: &mut App) {
    app.add_systems(Startup, setup)
        .add_systems(Update, sheep_system)
        .add_observer(observe_buttons);
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn(UiRoot);
}

#[derive(Component)]
struct UiRoot;

#[derive(Component)]
struct Sheep;

#[derive(Component, Default, Clone)]
enum SheepButton {
    #[default]
    Increment,
    Decrement,
}

// A query that pulls data from the ecs and then updates it using a template.
fn sheep_system(mut commands: Commands, sheep: Query<&Sheep>, root: Single<Entity, With<UiRoot>>) {
    let num_sheep = sheep.iter().len();

    let template = bsn! {
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(5.0),
        } [
            (:counter(num_sheep, "sheep", SheepButton::Increment, SheepButton::Decrement))
        ]
    };

    commands.entity(*root).retain_scene(template);
}

// A function that returns an ecs template.
fn counter<T: Component + Default + Clone>(num: usize, name: &str, inc: T, dec: T) -> impl Scene {
    let name = name.to_string();
    bsn! {
        Node [
            youhave: Text("You have ") [
                TextSpan(format!("{num}")),
                TextSpan(format!(" {name}!")),
            ],
            {1}: ( Button, Text("Increase"), TextColor(css::GREEN), {inc.clone()}, {visible_if(num < 100)} ),
            {2}: ( Button, Text("Decrease"), TextColor(css::RED), {dec.clone()}, {visible_if(num > 0)} ),
        ]
    }
}

// A component helper function for computing visibility.
fn visible_if(condition: bool) -> Visibility {
    if condition {
        Visibility::Visible
    } else {
        Visibility::Hidden
    }
}

// A global observer which responds to button clicks.
fn observe_buttons(
    mut trigger: Trigger<Pointer<Released>>,
    buttons: Query<&SheepButton>,
    sheep: Query<Entity, With<Sheep>>,
    mut commands: Commands,
) {
    match buttons.get(trigger.target).ok() {
        Some(SheepButton::Increment) => {
            commands.spawn(Sheep);
        }
        Some(SheepButton::Decrement) => {
            if let Some(sheep) = sheep.iter().next() {
                commands.entity(sheep).despawn();
            }
        }
        _ => {}
    }
    trigger.propagate(false);
}
