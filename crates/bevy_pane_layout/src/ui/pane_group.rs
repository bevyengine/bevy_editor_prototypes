use bevy::{
    ecs::{query::QueryEntityError, system::SystemParam},
    prelude::*,
    window::SystemCursorIcon,
    winit::cursor::CursorIcon,
};
use bevy_context_menu::{ContextMenu, ContextMenuOption};
use bevy_editor_styles::Theme;

use crate::prelude::PaneStructure;
use crate::{handlers::remove_pane_group, pane::Pane, Size};

use super::pane::{spawn_pane_node, PaneNode, SelectedPane};

#[derive(Component, Clone)]
pub(crate) struct PaneGroup {
    pub(crate) header: Entity,
    pub(crate) content: Entity,
    pub(crate) panes: Vec<Entity>,
    pub(crate) selected_pane: Option<usize>,
}

impl PaneGroup {
    fn new(header: Entity, content: Entity) -> Self {
        Self {
            header,
            content,
            panes: Vec::new(),
            selected_pane: None,
        }
    }
}

/// Node to add widgets into the header of a Pane Group.
#[derive(Component)]
pub struct PaneGroupHeaderNode;

/// Node to denote the area of the Pane Group.
#[derive(Component)]
pub struct PaneGroupAreaNode;

/// Node to denote the content section of the Pane Group.
#[derive(Component)]
pub struct PaneGroupContentNode;

/// A list of commands that will be run to modify a Pane Group.
pub struct PaneGroupCommands<'a> {
    entity: Entity,
    commands: Commands<'a, 'a>,
    header: Entity,
    content: Entity,
    panes: Vec<PaneStructure>,
    selected_pane: Option<usize>,
}

impl<'a> PaneGroupCommands<'a> {
    pub(crate) fn new(
        entity: Entity,
        commands: Commands<'a, 'a>,
        header: Entity,
        content: Entity,
    ) -> Self {
        Self {
            entity,
            commands,
            header,
            content,
            panes: Vec::new(),
            selected_pane: None,
        }
    }

    /// Adds a Pane to the end of this Pane Group.
    pub fn add_pane<T: Pane>(&mut self, theme: &Theme, pane: T) -> &'a mut PaneGroupCommands {
        let pane = spawn_pane_node(
            &mut self.commands,
            theme,
            pane,
            self.entity,
            self.header,
            self.content,
        );
        self.panes.push(pane);

        if self.panes.len() == 1 {
            self.set_selected_pane(0);
        }

        self.update_group();
        self
    }

    /// Sets the given Pane as the active pane
    pub fn select_pane(&mut self, index: usize) -> &'a mut PaneGroupCommands {
        if index >= self.panes.len() {
            warn!("Tried to select invalid Pane"); // Panic? Return Error?
            return self;
        };

        self.unselect_pane();
        self.set_selected_pane(index);

        self.update_group();
        self
    }

    /// Removes and despawns a pane in this group.
    pub fn remove_pane(&mut self, pane: Entity) -> &'a mut PaneGroupCommands {
        let index = self.panes.iter().position(|val| val.root == pane).unwrap();
        self.remove_pane_at(index)
    }

    /// Removes and despawns the pane in this group at the given index.
    pub fn remove_pane_at(&mut self, index: usize) -> &'a mut PaneGroupCommands {
        let pane = self.panes.remove(index);
        self.commands.entity(pane.container).despawn_recursive();
        self.commands.entity(pane.header_tag).despawn_recursive();

        if let Some(selected) = self.selected_pane {
            if selected > index {
                self.selected_pane = Some(selected - 1);
            } else if selected == index && self.panes.len() > 0 {
                self.set_selected_pane(0);
            }
        }

        self.update_group();
        self
    }

    /// Returns the panes in the selected group
    pub fn panes(&self) -> &Vec<PaneStructure> {
        &self.panes
    }

    pub(crate) fn group(&self) -> PaneGroup {
        PaneGroup {
            header: self.header,
            content: self.content,
            panes: self.panes.iter().map(|pane| pane.root).collect(),
            selected_pane: self.selected_pane.clone(),
        }
    }

    // Selects a given pane, without checking bounds. Does not update the Pane Group.
    fn set_selected_pane(&mut self, index: usize) {
        self.selected_pane = Some(index);
        self.commands
            .entity(self.panes[index].root)
            .insert(SelectedPane);
    }

    // Unselects the selected pane, if one exists. Does not update the Pane Group.
    fn unselect_pane(&mut self) {
        if let Some(cur_index) = self.selected_pane {
            self.commands
                .entity(self.panes[cur_index].root)
                .remove::<SelectedPane>();

            self.selected_pane = None;
        };
    }

    fn update_group(&mut self) {
        let group_info = self.group();
        self.commands.entity(self.entity).insert(group_info);
    }
}

/// A system parameter that can be used to get divider commands for each Divider Entity.
#[derive(SystemParam)]
pub struct PaneGroupCommandsQuery<'w, 's> {
    commands: Commands<'w, 's>,
    pane_groups: Query<'w, 's, &'static PaneGroup>,
    panes: Query<'w, 's, &'static PaneNode>,
}

impl<'w, 's> PaneGroupCommandsQuery<'w, 's> {
    /// Gets the DividerCommands for a given entity.
    pub fn get(&mut self, entity: Entity) -> Result<PaneGroupCommands, QueryEntityError> {
        let pane_group = self.pane_groups.get(entity)?;
        let panes = pane_group
            .panes
            .iter()
            .map(|val| {
                let node = self.panes.get(*val).unwrap();
                PaneStructure {
                    container: node.container,
                    root: *val,
                    header_tag: node.header,
                }
            })
            .collect();

        Ok(PaneGroupCommands {
            entity,
            commands: self.commands.reborrow(),
            header: pane_group.header,
            content: pane_group.content,
            panes,
            selected_pane: pane_group.selected_pane,
        })
    }
}

pub(crate) fn spawn_pane_group<'a>(
    commands: &'a mut Commands,
    theme: &Theme,
    size: f32,
) -> (EntityCommands<'a>, PaneGroup) {
    // Unstyled root node
    let root = commands
        .spawn((
            Name::new("Pane Group"),
            Node {
                padding: UiRect::all(Val::Px(1.5)),
                ..default()
            },
            Size(size),
        ))
        .id();

    // Area
    let area = commands
        .spawn((
            Name::new("Pane Group Area"),
            Node {
                overflow: Overflow::clip(),
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            theme.pane.area_background_color,
            theme.general.border_radius,
            PaneGroupAreaNode,
        ))
        .set_parent(root)
        .id();

    // Header
    let header = commands
        .spawn((
            Name::new("Pane Group Header"),
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(6.0),
                padding: UiRect::axes(Val::Px(5.), Val::Px(3.)),
                width: Val::Percent(100.),
                height: Val::Px(27.),
                align_items: AlignItems::Center,
                flex_shrink: 0.,
                ..default()
            },
            theme.pane.header_background_color,
            theme.pane.header_border_radius,
            ContextMenu::new([ContextMenuOption::new(
                "Close Group",
                move |mut commands, _entity| {
                    commands.run_system_cached_with(remove_pane_group, root);
                },
            )]),
            PaneGroupHeaderNode,
        ))
        .observe(
            move |_trigger: Trigger<Pointer<Move>>,
                  window_query: Query<Entity, With<Window>>,
                  mut commands: Commands| {
                let window = window_query.single();
                commands
                    .entity(window)
                    .insert(CursorIcon::System(SystemCursorIcon::Pointer));
            },
        )
        .observe(
            |_trigger: Trigger<Pointer<Out>>,
             window_query: Query<Entity, With<Window>>,
             mut commands: Commands| {
                let window = window_query.single();
                commands
                    .entity(window)
                    .insert(CursorIcon::System(SystemCursorIcon::Default));
            },
        )
        .set_parent(area)
        .id();

    // Content
    let content = commands
        .spawn((
            Name::new("Pane Group Content"),
            Node {
                flex_grow: 1.,
                ..default()
            },
            PaneGroupContentNode,
        ))
        .set_parent(area)
        .id();

    let pane_group_data = PaneGroup::new(header, content);
    commands.entity(root).insert(pane_group_data.clone());

    (commands.entity(root), pane_group_data)
}
