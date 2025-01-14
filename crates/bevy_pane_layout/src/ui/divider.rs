use bevy::{
    ecs::{query::QueryEntityError, system::SystemParam},
    prelude::*,
};
use bevy_editor_styles::Theme;

use crate::{Divider, Size};

use super::{
    pane_group::{spawn_pane_group, PaneGroupCommands},
    resize_handle::spawn_resize_handle,
};

/// A list of commands that will be run to modify a Divider Entity.
pub struct DividerCommands<'a> {
    entity: Entity,
    commands: Commands<'a, 'a>,
    direction: Divider,
    contents: Vec<Entity>,
    resize_handles: Vec<Entity>,
}

impl<'a> DividerCommands<'a> {
    pub(crate) fn new(entity: Entity, commands: Commands<'a, 'a>, divider: Divider) -> Self {
        DividerCommands {
            entity,
            commands,
            direction: divider,
            contents: Vec::new(),
            resize_handles: Vec::new(),
        }
    }

    /// Adds a divider to this divider, returns a DividerCommands object for the new divider.
    pub fn add_divider(&mut self, size: f32) -> DividerCommands {
        // If a respawn handle is needed, add it
        if self.contents.len() != 0 {
            self.add_resize_handle();
        }

        // Create sub divider
        let new_dir = self.direction.flipped();
        let child = create_divider(&mut self.commands, new_dir, size)
            .set_parent(self.entity)
            .id();
        self.contents.push(child);

        DividerCommands::new(child, self.commands.reborrow(), new_dir)
    }

    /// Adds a Pane Group to this divider, return the new Pane Group's commands.
    pub fn add_pane_group(&mut self, theme: &Theme, size: f32) -> PaneGroupCommands {
        // If a respawn handle is needed, add it
        if self.contents.len() != 0 {
            self.add_resize_handle();
        }

        let (mut pane_group, data) = spawn_pane_group(&mut self.commands, theme, size);
        pane_group.set_parent(self.entity);
        self.contents.push(pane_group.id());

        PaneGroupCommands::new(
            pane_group.id(),
            self.commands.reborrow(),
            data.header,
            data.content,
        )
    }

    /// Removes and despawns the given content entity.
    pub fn remove(&mut self, entity: Entity) {
        let i = self.contents.iter().position(|val| *val == entity).unwrap();
        self.remove_at(i)
    }

    /// Removes and despawns the content element in the ith position. Note, this is not the necesarily the ith child
    /// of the divider entity (because resize handles are inserted between the content entities).
    pub fn remove_at(&mut self, index: usize) {
        let group_id = self.contents.remove(index);
        self.commands.entity(group_id).despawn_recursive();

        if self.resize_handles.len() != 0 {
            let index = if index >= self.resize_handles.len() {
                self.resize_handles.len() - 1
            } else {
                index
            };

            self.commands
                .entity(self.resize_handles[index])
                .despawn_recursive();
            self.resize_handles.remove(index);
        }
    }

    fn add_resize_handle(&mut self) {
        let handle = spawn_resize_handle(&mut self.commands, self.direction)
            .set_parent(self.entity)
            .id();
        self.resize_handles.push(handle);
    }
}

/// A system parameter that can be used to get divider commands for each Divider Entity.
#[derive(SystemParam)]
pub struct DividerCommandsQuery<'w, 's> {
    commands: Commands<'w, 's>,
    dividers: Query<'w, 's, (&'static Divider, &'static Children)>,
    sizes: Query<'w, 's, (), With<Size>>,
}

impl<'w, 's> DividerCommandsQuery<'w, 's> {
    /// Gets the DividerCommands for a given entity.
    pub fn get(&mut self, entity: Entity) -> Result<DividerCommands, QueryEntityError> {
        let (divider, children) = self.dividers.get(entity)?;
        let mut contents = Vec::new();
        let mut resize_handles = Vec::new();

        for child in children.iter() {
            if self.sizes.contains(*child) {
                contents.push(*child);
            } else {
                // Non contents should be resize handles, we could check this in the future
                resize_handles.push(*child);
            }
        }

        Ok(DividerCommands {
            entity,
            commands: self.commands.reborrow(),
            direction: divider.clone(),
            contents,
            resize_handles,
        })
    }
}

pub(crate) fn create_divider<'a>(
    commands: &'a mut Commands,
    divider: Divider,
    size: f32,
) -> EntityCommands<'a> {
    commands.spawn((
        Name::new("Divider"),
        Node {
            flex_direction: match divider {
                Divider::Horizontal => FlexDirection::Row,
                Divider::Vertical => FlexDirection::Column,
            },
            ..default()
        },
        Size(size),
        divider,
    ))
}

/// The entry point for a layout. Creates a divider that panes can be added to.
pub fn spawn_root_divider<'a>(
    commands: &'a mut Commands,
    divider: Divider,
    parent: Option<Entity>,
    size: f32,
) -> DividerCommands<'a> {
    let mut child = create_divider(commands, divider, size);
    if let Some(parent) = parent {
        child.set_parent(parent);
    };
    DividerCommands::new(child.id(), commands.reborrow(), divider)
}
