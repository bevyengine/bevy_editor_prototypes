//! This example demonstrates how to use the collapsing header widget with EntityDiffTree.

use bevy::prelude::*;
use bevy_collapsing_header::*;
use bevy_incomplete_bsn::entity_diff_tree::{DiffTreeCommands, EntityDiffTree};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CollapsingHeaderPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d::default());

    let tree = EntityDiffTree::new()
        .with_patch_fn(|header: &mut CollapsingHeader| {
            header.text = "Hello, collapsing header!".to_string();
        })
        .with_patch_fn(|color: &mut BackgroundColor| {
            *color = BackgroundColor(Color::srgb(0.1, 0.1, 0.1));
        })
        .with_child(EntityDiffTree::new().with_patch_fn(|text: &mut Text| {
            text.0 = "Content 1".to_string();
        }))
        .with_child(EntityDiffTree::new().with_patch_fn(|text: &mut Text| {
            text.0 = "Content 2".to_string();
        }))
        .with_child(EntityDiffTree::new().with_patch_fn(|text: &mut Text| {
            text.0 = "Content 3".to_string();
        }));

    commands.spawn_empty().diff_tree(tree);
}
