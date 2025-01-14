//! Resizable, divider-able panes for Bevy.

mod handlers;
pub mod pane;
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
use bevy::{prelude::*, utils::HashSet};
use bevy_editor_styles::Theme;

use crate::registry::PaneRegistryPlugin;

/// Crate prelude.
pub mod prelude {
    pub use crate::pane::Pane;
    pub use crate::registry::PaneAppExt;
    pub use crate::ui::divider::{spawn_root_divider, DividerCommands, DividerCommandsQuery};
    pub use crate::ui::pane::PaneStructure;
    pub use crate::ui::pane_group::{
        PaneGroupAreaNode, PaneGroupCommands, PaneGroupContentNode, PaneGroupHeaderNode,
    };
    pub use crate::{Divider, RootPaneLayoutNode};
}

/// The Bevy Pane Layout Plugin.
pub struct PaneLayoutPlugin;

impl Plugin for PaneLayoutPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PaneRegistryPlugin)
            .init_resource::<DragState>()
            .add_systems(Startup, setup_background.in_set(PaneLayoutSet))
            .add_systems(
                Update,
                (normalize_size, apply_size).chain().in_set(PaneLayoutSet),
            );
    }
}

/// Makes sure the sizes for each divider's contents adds up to 1
fn normalize_size(
    mut size_query: Query<(Mut<Size>, &Parent)>,
    divider_query: Query<(Entity, &Divider, Ref<Children>)>,
) {
    let mut changed_dividers = HashSet::new();

    for (size, parent) in size_query.iter() {
        if size.is_changed() {
            changed_dividers.insert(parent.get());
        }
    }

    for (entity, _, children) in divider_query.iter() {
        if children.is_changed() {
            changed_dividers.insert(entity);
        }
    }

    for entity in changed_dividers.iter() {
        let Ok((.., div_children)) = divider_query.get(*entity) else {
            continue;
        };

        let mut total_size = 0.0;
        for child in div_children.iter() {
            if let Ok((size, _)) = size_query.get(*child) {
                total_size += size.0;
            }
        }

        if total_size != 1.0 {
            for child in div_children.iter() {
                if let Ok((mut size, _)) = size_query.get_mut(*child) {
                    size.0 /= total_size;
                }
            }
        }
    }
}

/// Updates the Node's size to match the Size component
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
fn setup_background(
    mut commands: Commands,
    theme: Res<Theme>,
    panes_root: Single<Entity, With<RootPaneLayoutNode>>,
) {
    commands.entity(*panes_root).insert((
        Node {
            padding: UiRect::all(Val::Px(1.)),
            width: Val::Percent(100.),
            height: Val::Px(0.0),
            flex_grow: 1.0,
            ..default()
        },
        theme.general.background_color,
    ));
}

// TODO: Reimplement
/// Removes a divider from the hierarchy when it has only one child left, replacing itself with that child.
// fn cleanup_divider_single_child(
//     mut commands: Commands,
//     mut query: Query<(Entity, &Divider, &Parent), Changed<Children>>,
//     mut size_query: Query<&mut Size>,
//     children_query: Query<&Children>,
//     resize_handle_query: Query<(), With<ResizeHandle>>,
// ) {
//     for (entity, children, parent) in &mut query {
//         let mut iter = children
//             .iter()
//             .filter(|child| !resize_handle_query.contains(**child));
//         let child = *iter.next().unwrap();
//         if iter.next().is_some() {
//             continue;
//         }

//         let size = size_query.get(entity).unwrap().0;
//         size_query.get_mut(child).unwrap().0 = size;

//         // Find the index of this divider among its siblings
//         let siblings = children_query.get(parent.get()).unwrap();
//         let index = siblings.iter().position(|s| *s == entity).unwrap();

//         commands
//             .entity(parent.get())
//             .insert_children(index, &[child]);
//         commands.entity(entity).despawn_recursive();
//     }
// }

// I would prefer the divider component be private, but it is currently used by the spawn_root_divider function.
/// A node that divides an area into multiple areas along an axis.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum Divider {
    /// A divider that stacks its contents horizontally
    Horizontal,
    /// A divider that stacks its contents vertically
    Vertical,
}

impl Divider {
    /// Gets the reversed direction of the divider
    pub fn flipped(&self) -> Self {
        match self {
            Divider::Horizontal => Divider::Vertical,
            Divider::Vertical => Divider::Horizontal,
        }
    }
}

#[derive(Component)]
struct ResizeHandle;

/// The fraction of space this element takes up in the [`Divider`] it's a child of.
#[derive(Component)]
struct Size(f32);

/// Root node to capture all editor UI elements, nothing but the layout system should modify this.
#[derive(Component)]
pub struct RootPaneLayoutNode;
