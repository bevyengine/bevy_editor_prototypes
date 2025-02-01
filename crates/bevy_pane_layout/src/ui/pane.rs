use bevy::prelude::*;
use bevy_context_menu::{ContextMenu, ContextMenuOption};
use bevy_editor_styles::Theme;

use crate::{handlers::remove_pane, pane::Pane};

use super::pane_group::PaneGroupCommandsQuery;

/// Tag applied to Panes that are currently active.
#[derive(Component)]
pub struct SelectedPane;

/// Tag applied to a pane container. (Should this be kept?)
#[derive(Component)]
pub struct PaneContainer;

/// Root node for each pane.
#[derive(Component)]
pub struct PaneNode {
    pub(crate) id: String,
    pub(crate) header: Entity,
    pub(crate) header_text: Entity,
    pub(crate) container: Entity,
    pub(crate) group: Entity,
}

impl PaneNode {
    pub(crate) fn new(
        id: String,
        header: Entity,
        header_text: Entity,
        container: Entity,
        group: Entity,
    ) -> Self {
        Self {
            id,
            header,
            header_text,
            container,
            group,
        }
    }
}

/// The node structure of a pane.
#[derive(Clone, Copy)]
pub struct PaneStructure {
    /// The root of the pane.
    pub root: Entity,
    /// The container holding the root of the pane.
    pub(crate) container: Entity,
    /// The header node. Child of the area node.
    pub header_tag: Entity,
}

impl PaneStructure {
    pub(crate) fn new(root: Entity, container: Entity, header_tag: Entity) -> Self {
        Self {
            root,
            container,
            header_tag,
        }
    }
}

/// Root node for each pane.
#[derive(Component)]
pub struct PaneHeaderTab {
    pane: Entity,
}

impl PaneHeaderTab {
    fn new(pane: Entity) -> Self {
        Self { pane }
    }
}

pub(crate) fn spawn_pane_node<T: Pane>(
    commands: &mut Commands,
    theme: &Theme,
    pane: T,
    group_id: Entity,
    group_header: Entity,
    group_content: Entity,
) -> PaneStructure {
    let pane_entity = commands.spawn_empty().id();

    let pane_header = commands
        .spawn((
            Name::new(format!("{} Header", T::NAME)),
            BorderRadius::all(Val::Px(3.0)),
            Node {
                flex_shrink: 0.0,
                padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(Color::hsla(0., 0., 0., 0.)),
            ContextMenu::new([ContextMenuOption::new(
                "Close Pane",
                move |mut commands, _entity| {
                    commands.run_system_cached_with(remove_pane, pane_entity);
                },
            )]),
        ))
        .set_parent(group_header)
        .observe(
            |trigger: Trigger<Pointer<Up>>,
             tab_query: Query<&PaneHeaderTab>,
             pane_query: Query<&PaneNode>,
             mut pane_commands_query: PaneGroupCommandsQuery| {
                if trigger.button == PointerButton::Primary {
                    let pane_entity = tab_query.get(trigger.entity()).unwrap().pane;
                    let pane_info = pane_query.get(pane_entity).unwrap();
                    let mut pane_commands = pane_commands_query.get(pane_info.group).unwrap();
                    let index = pane_commands
                        .panes()
                        .iter()
                        .position(|val| val.root == pane_entity);

                    if let Some(index) = index {
                        pane_commands.select_pane(index);
                    } else {
                        warn!("Failed to select Pane.")
                    }
                }
            },
        )
        .id();

    let pane_header_text = commands
        .spawn((
            Text::new(Into::<String>::into(T::NAME)),
            TextFont {
                font: theme.text.font.clone(),
                font_size: 11.,
                ..default()
            },
            TextColor(theme.text.low_priority),
        ))
        .set_parent(pane_header)
        .id();

    // This exists between the pane group content node and the pane root so that we can toggle the Pane's visibility without interfering with its contents
    let pane_container = commands
        .spawn((
            Name::new(format!("{} Pane Container", T::NAME)),
            Node {
                display: Display::None, // Panes start unselected/invisible
                flex_grow: 1.,
                ..default()
            },
            PaneContainer,
        ))
        .set_parent(group_content)
        .id();

    commands
        .entity(pane_entity)
        .insert((
            Name::new(format!("{} Pane", T::NAME)),
            Node {
                display: Display::Flex,
                flex_grow: 1.,
                ..default()
            },
            PaneNode::new(
                T::ID.into(),
                pane_header,
                pane_header_text,
                pane_container,
                group_id,
            ),
            pane,
        ))
        .observe(
            |trigger: Trigger<OnAdd, SelectedPane>,
             panes: Query<&PaneNode>,
             mut containers: Query<&mut Node, With<PaneContainer>>,
             mut backgrounds: Query<&mut BackgroundColor, With<PaneHeaderTab>>,
             mut text_colors: Query<&mut TextColor>,
             theme: Res<Theme>| {
                let pane = panes.get(trigger.entity()).unwrap();

                let mut node = containers.get_mut(pane.container).unwrap();
                node.display = Display::Flex;

                let mut header_background = backgrounds.get_mut(pane.header).unwrap();
                header_background.0 = theme.pane.header_tab_background_color.0;

                let mut header_tab_text = text_colors.get_mut(pane.header_text).unwrap();
                header_tab_text.0 = theme.text.text_color;
            },
        )
        .observe(
            |trigger: Trigger<OnRemove, SelectedPane>,
             panes: Query<&PaneNode>,
             mut containers: Query<&mut Node, With<PaneContainer>>,
             mut backgrounds: Query<&mut BackgroundColor, With<PaneHeaderTab>>,
             mut text_colors: Query<&mut TextColor>,
             theme: Res<Theme>| {
                let pane = panes.get(trigger.entity()).unwrap();

                if let Ok(mut node) = containers.get_mut(pane.container) {
                    node.display = Display::None;
                }

                if let Ok(mut header_background) = backgrounds.get_mut(pane.header) {
                    header_background.0 = Color::hsla(0., 0., 0., 0.);
                }

                if let Ok(mut header_tab_text) = text_colors.get_mut(pane.header_text) {
                    header_tab_text.0 = theme.text.low_priority;
                }
            },
        )
        .set_parent(pane_container);

    commands
        .entity(pane_header)
        .insert(PaneHeaderTab::new(pane_entity));

    PaneStructure::new(pane_entity, pane_container, pane_header)
}
