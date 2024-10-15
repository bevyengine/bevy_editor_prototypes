#![allow(missing_docs)]
//! Resizable, divider-able panes for Bevy.

mod handlers;
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

use crate::ui::{spawn_divider, spawn_pane, spawn_resize_handle};

/// The Bevy Pane Layout Plugin.
pub struct PaneLayoutPlugin;

impl Plugin for PaneLayoutPlugin {
    fn build(&self, app: &mut App) {
        let mut pane_registry = app.world_mut().get_resource_or_init::<PaneRegistry>();

        // TODO Move these registrations to their respective crates.
        pane_registry.register(Pane::new("Properties", |mut _commands, _pane_root| {
            // Todo
        }));

        pane_registry.register(Pane::new("Scene Tree", |mut _commands, _pane_root| {
            // Todo
        }));

        app.init_resource::<DragState>()
            .init_resource::<PaneRegistry>()
            .add_systems(Startup, setup.in_set(PaneLayoutSet))
            .add_systems(
                Update,
                (
                    (
                        cleanup_divider_single_child,
                        |mut query: Query<(&Size, &mut Style), Changed<Size>>| {
                            // TODO check if ordering issue is fixed

                            for (size, mut style) in &mut query {
                                style.flex_basis = Val::Percent(size.0 * 100.);
                            }
                        },
                    )
                        .chain(),
                    on_pane_creation,
                )
                    .in_set(PaneLayoutSet),
            );
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

fn on_pane_creation(
    mut query: Query<(Entity, &PaneRootNode), Added<PaneRootNode>>,
    mut pane_registry: ResMut<PaneRegistry>,
    mut commands: Commands,
) {
    for (entity, pane_root) in &mut query {
        let pane = pane_registry
            .panes
            .iter_mut()
            .find(|pane| pane.name == pane_root.name);

        if let Some(pane) = pane {
            (pane.creation_callback)(commands.reborrow(), entity);
        } else {
            warn!(
                "No pane found in the registry with name: '{}'",
                pane_root.name
            );
        }
    }
}

/// System Set to set up the Pane Layout.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PaneLayoutSet;

#[derive(Resource, Default)]
pub struct PaneRegistry {
    panes: Vec<Pane>,
}

impl PaneRegistry {
    pub fn register(&mut self, pane: Pane) {
        self.panes.push(pane);
    }
}

pub struct Pane {
    name: String,
    creation_callback: Box<dyn FnMut(Commands, Entity) + Send + Sync>,
}

impl Pane {
    pub fn new(
        name: impl Into<String>,
        creation_callback: impl FnMut(Commands, Entity) + Send + Sync + 'static,
    ) -> Self {
        Pane {
            name: name.into(),
            creation_callback: Box::new(creation_callback),
        }
    }
}

// TODO There is no way to save or load layouts at this moment.
// The setup system currently just creates a default layout at startup.
fn setup(
    mut commands: Commands,
    theme: Res<Theme>,
    panes_root: Single<Entity, With<RootPaneLayoutNode>>,
) {
    commands.entity(*panes_root).insert(NodeBundle {
        background_color: theme.background_color,
        style: Style {
            padding: UiRect::all(Val::Px(1.)),
            height: Val::Percent(100.),
            width: Val::Percent(100.),
            ..default()
        },
        ..default()
    });

    let divider = spawn_divider(&mut commands, Divider::Horizontal, 1.)
        .set_parent(*panes_root)
        .id();

    // spawn_pane(&mut commands, &theme, 0.2, "Scene Tree").set_parent(divider);

    let sub_divider = spawn_divider(&mut commands, Divider::Vertical, 0.2)
        .set_parent(divider)
        .id();

    spawn_pane(&mut commands, &theme, 0.4, "Scene Tree").set_parent(sub_divider);
    spawn_resize_handle(&mut commands, Divider::Vertical).set_parent(sub_divider);
    spawn_pane(&mut commands, &theme, 0.6, "Properties").set_parent(sub_divider);

    spawn_resize_handle(&mut commands, Divider::Horizontal).set_parent(divider);
    spawn_pane(&mut commands, &theme, 0.8, "Viewport 3D").set_parent(divider);
}

#[derive(Component)]
struct ResizeHandle;

// fn add_dividers(
//     mut commands: Commands,
//     mut query: Query<(Entity, &Children), (Changed<Children>, With<Divider>)>,
//     resize_handle_query: Query<(), With<ResizeHandle>>,
// ) {
//     for (entity, children) in &mut query {
//         for i in (0..children.len()).rev() {
//             commands.entity(*child).despawn_recursive();
//         }
//     }
// }

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
