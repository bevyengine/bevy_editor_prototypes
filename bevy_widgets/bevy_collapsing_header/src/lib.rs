//! This crate provides a collapsing header widget for Bevy.

pub use bevy::prelude::*;
use bevy_editor_styles::Theme;
use bevy_incomplete_bsn::{children_patcher::*, entity_diff_tree::DiffTree};

/// A plugin that adds collapsing header functionality to the Bevy UI.
pub struct CollapsingHeaderPlugin;

impl Plugin for CollapsingHeaderPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CollapsingHeader>();

        app.add_observer(on_header_click);
        app.add_systems(PreUpdate, on_header_change);
        app.add_systems(PreUpdate, update_header_font);
    }
}

/// A component that represents a collapsing header widget.
///
/// This struct provides functionality for creating and managing a collapsible header
/// in a Bevy UI. It allows for toggling the visibility of its child elements.
#[derive(Component, Reflect, Clone)]
#[reflect(Component, ChildrenPatcher, Default)]
#[require(Node)]
pub struct CollapsingHeader {
    /// The text to display in the header.
    pub text: String,
    /// A boolean flag indicating whether the header is collapsed (true) or expanded (false).
    pub is_collapsed: bool,
}

impl Default for CollapsingHeader {
    fn default() -> Self {
        Self {
            is_collapsed: true,
            text: "".to_string(),
        }
    }
}

impl CollapsingHeader {
    /// Create a new collapsing header with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_collapsed: true,
        }
    }
}

impl ChildrenPatcher for CollapsingHeader {
    fn children_patch(&mut self, children: &mut Vec<DiffTree>) {

        info!("Patching children for header {:?}. Children count: {}", &self.text, children.len());

        let move_text = self.text.clone();
        let collapsed = self.is_collapsed;
        let header = DiffTree::new()
            .with_patch_fn(move |text: &mut Text| {
                let pred = if collapsed {
                    format!("+ {}", move_text.clone())
                } else {
                    format!("- {}", move_text.clone())
                };
                text.0 = pred;
            })
            .with_patch_fn(|_: &mut CollapsingHeaderText| {});

        let collapsed = self.is_collapsed;
        let mut collapsable = DiffTree::new()
            .with_patch_fn(move |node: &mut Node| {
                if collapsed {
                    node.height = Val::Px(0.0);
                } else {
                    node.height = Val::Auto;
                }
                node.display = Display::Flex;
                node.flex_direction = FlexDirection::Column;
                node.overflow = Overflow::clip();
            })
            .with_patch_fn(|_: &mut CollapsingHeaderContent| {});

        for child in children.drain(..) {
            collapsable.children.push(child);
        }

        children.push(header);
        children.push(collapsable);
    }
}

#[derive(Component, Default, Clone)]
pub struct CollapsingHeaderText;

#[derive(Component, Default, Clone)]
pub struct CollapsingHeaderContent;

fn on_header_click(
    trigger: Trigger<Pointer<Click>>,
    mut q_headers: Query<&mut CollapsingHeader>,
    q_parents: Query<&Parent, With<CollapsingHeaderText>>,
) {
    let entity = trigger.entity();
    let Ok(header_entity) = q_parents.get(entity).map(|p| p.get()) else {
        return;
    };

    let Ok(mut header) = q_headers.get_mut(header_entity) else {
        return;
    };

    header.is_collapsed = !header.is_collapsed;
}

fn on_header_change(
    mut q_nodes: Query<&mut Node>,
    mut q_texts: Query<&mut Text>,
    q_changed: Query<(Entity, &CollapsingHeader, &Children), Changed<CollapsingHeader>>,
) {
    for (entity, header, children) in q_changed.iter() {
        {
            let Ok(mut header_node) = q_nodes.get_mut(entity) else {
                continue;
            };

            header_node.display = Display::Flex;
            header_node.flex_direction = FlexDirection::Column;
        }

        {
            let Ok(mut content_node) = q_nodes.get_mut(children[1]) else {
                continue;
            };

            if header.is_collapsed {
                content_node.height = Val::Px(0.0);
            } else {
                content_node.height = Val::Auto;
            }
            content_node.display = Display::Flex;
            content_node.flex_direction = FlexDirection::Column;
            content_node.overflow = Overflow::clip();
        }

        let Ok(mut text) = q_texts.get_mut(children[0]) else {
            continue;
        };

        let pred = if header.is_collapsed {
            format!("+ {}", header.text.clone())
        } else {
            format!("- {}", header.text.clone())
        };
        text.0 = pred;
    }
}

fn update_header_font(
    mut q_changed: Query<
        (&mut TextFont, &mut TextColor),
        (Changed<Text>, With<CollapsingHeaderText>),
    >,
    theme: Res<Theme>,
) {
    for (mut font, mut color) in q_changed.iter_mut() {
        font.font = theme.text.font.clone();
        color.0 = theme.text.text_color;
    }
}
