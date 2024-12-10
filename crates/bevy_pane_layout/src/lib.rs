//! Resizable, divider-able panes for Bevy.

mod handlers;
mod pane_drop_area;
pub mod registry;
mod ui;

/// The Bevy Pane Layout system.
/// The intent of this system is to provide a way to create resizable, split-able panes in Bevy.
/// Mimicking the behavior of of Blender's layout system.
///
/// Blender Documentation: <https://docs.blender.org/manual/en/latest/interface/window_system/areas.html>
///
/// Requirements for a valid Pane:
/// - All panes must fit within their bounds, no overflow is allowed.
/// - Panes can not have power over the layout system, their dimensions are controlled by the layout system and should not be modified by anything else.
/// - All panes must have a header, a content area, however a footer is optional.
/// - Panes cannot have min/max sizes, they must be able to be resized to any size.
///   - If a pane can not be sensibly resized, it can overflow under the other panes.
/// - Panes must not interfere with each other, only temporary/absolute positioned elements are allowed to overlap panes.
use bevy::prelude::*;
use bevy_editor_styles::Theme;

use crate::{
    registry::{PaneAppExt, PaneRegistryPlugin, PaneStructure},
    ui::{spawn_divider, spawn_pane, spawn_resize_handle},
};

/// Crate prelude.
pub mod prelude {
    pub use crate::{
        registry::{PaneAppExt, PaneStructure},
        PaneAreaNode, PaneContentNode, PaneHeaderNode,
    };
}

/// The Bevy Pane Layout Plugin.
pub struct PaneLayoutPlugin;

impl Plugin for PaneLayoutPlugin {
    fn build(&self, app: &mut App) {
        // TODO Move these registrations to their respective crates.
        app.register_pane("Properties", |_pane_structure: In<PaneStructure>| {
            // Todo
        });

        app.add_plugins(PaneRegistryPlugin)
            .init_resource::<DragState>()
            .add_systems(Startup, setup.in_set(PaneLayoutSet))
            .add_systems(
                Update,
                (cleanup_divider_single_child, apply_size)
                    .chain()
                    .in_set(PaneLayoutSet),
            );
    }
}

fn apply_size(
    mut query: Query<(Entity, &Size, &mut Node), Changed<Size>>,
    divider_query: Query<&Divider>,
    parent_query: Query<&Parent>,
) {
    for (entity, size, mut style) in &mut query {
        let parent = parent_query.get(entity).unwrap().get();
        let Ok(e) = divider_query.get(parent) else {
            style.width = Val::Percent(100.);
            style.height = Val::Percent(100.);
            continue;
        };

        match e {
            Divider::Horizontal => {
                style.width = Val::Percent(size.0 * 100.);
                style.height = Val::Percent(100.);
            }
            Divider::Vertical => {
                style.width = Val::Percent(100.);
                style.height = Val::Percent(size.0 * 100.);
            }
        }
    }
}

#[derive(Resource, Default)]
struct DragState {
    is_dragging: bool,
    offset: f32,
    min: f32,
    max: f32,
    parent_node_size: f32,
}

/// System Set to set up the Pane Layout.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PaneLayoutSet;

// TODO There is no way to save or load layouts at this moment.
// The setup system currently just creates a default layout at startup.
fn setup(
    mut commands: Commands,
    theme: Res<Theme>,
    panes_root: Single<Entity, With<RootPaneLayoutNode>>,
) {
    commands.entity(*panes_root).insert((
        Node {
            padding: UiRect::all(Val::Px(1.)),
            flex_grow: 1.,
            width: Val::Percent(100.),

            ..default()
        },
        theme.general.background_color,
    ));

    let divider = spawn_divider(&mut commands, Divider::Horizontal, 1.)
        .set_parent(*panes_root)
        .id();

    let sub_divider = spawn_divider(&mut commands, Divider::Vertical, 0.2)
        .set_parent(divider)
        .id();

    spawn_pane(&mut commands, &theme, 0.4, "Scene Tree").set_parent(sub_divider);
    spawn_resize_handle(&mut commands, Divider::Vertical).set_parent(sub_divider);
    spawn_pane(&mut commands, &theme, 0.6, "Properties").set_parent(sub_divider);

    spawn_resize_handle(&mut commands, Divider::Horizontal).set_parent(divider);

    let asset_browser_divider = spawn_divider(&mut commands, Divider::Vertical, 0.8)
        .set_parent(divider)
        .id();

    spawn_pane(&mut commands, &theme, 0.70, "Viewport 3D").set_parent(asset_browser_divider);
    spawn_resize_handle(&mut commands, Divider::Vertical).set_parent(asset_browser_divider);
    spawn_pane(&mut commands, &theme, 0.30, "Asset Browser").set_parent(asset_browser_divider);
}

/// Removes a divider from the hierarchy when it has only one child left, replacing itself with that child.
fn cleanup_divider_single_child(
    mut commands: Commands,
    mut query: Query<(Entity, &Children, &Parent), (Changed<Children>, With<Divider>)>,
    mut size_query: Query<&mut Size>,
    children_query: Query<&Children>,
    resize_handle_query: Query<(), With<ResizeHandle>>,
) {
    for (entity, children, parent) in &mut query {
        let mut iter = children
            .iter()
            .filter(|child| !resize_handle_query.contains(**child));
        let child = *iter.next().unwrap();
        if iter.next().is_some() {
            continue;
        }

        let size = size_query.get(entity).unwrap().0;
        size_query.get_mut(child).unwrap().0 = size;

        // Find the index of this divider among its siblings
        let siblings = children_query.get(parent.get()).unwrap();
        let index = siblings.iter().position(|s| *s == entity).unwrap();

        commands
            .entity(parent.get())
            .insert_children(index, &[child]);
        commands.entity(entity).despawn_recursive();
    }
}

/// A node that divides an area into multiple areas along an axis.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum Divider {
    Horizontal,
    Vertical,
}

#[derive(Component)]
struct ResizeHandle;

/// The fraction of space this element takes up in the [`Divider`] it's a child of.
#[derive(Component)]
struct Size(f32);

/// Root node to capture all editor UI elements, nothing but the layout system should modify this.
#[derive(Component)]
pub struct RootPaneLayoutNode;

/// Root node for each pane, holds all event nodes for layout and the basic structure for all Panes.
#[derive(Component)]
struct PaneRootNode {
    name: String,
}

/// Node to denote the area of the Pane.
#[derive(Component)]
pub struct PaneAreaNode;

/// Node to add widgets into the header of a Pane.
#[derive(Component)]
pub struct PaneHeaderNode;

/// Node to denote the content space of the Pane.
#[derive(Component)]
pub struct PaneContentNode;
