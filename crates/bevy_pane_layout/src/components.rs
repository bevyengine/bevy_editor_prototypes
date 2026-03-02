//! Pane UI components module.

use bevy::{
    ecs::template::template,
    feathers::theme::ThemeBackgroundColor,
    prelude::*,
    scene2::{Scene, bsn},
};

use crate::containers::{pane, pane_body, pane_header, tokens};
use crate::{PaneContentNode, PaneHeaderNode, ui::header_context_menu};

/// A standard editor pane.
pub fn editor_pane() -> impl Scene {
    bsn! {
        :pane
        :fit_to_parent
        Node {
            margin: UiRect::all(Val::Px(1.)),
        }
    }
}

/// A standard editor pane header.
pub fn editor_pane_header() -> impl Scene {
    bsn! {
        :pane_header
        PaneHeaderNode
        Node {
            flex_shrink: 0.,
        }
        template(|_| Ok(header_context_menu()))
    }
}

/// A standard editor pane body.
pub fn editor_pane_body() -> impl Scene {
    bsn! {
        :pane_body
        PaneContentNode
        Node {
            flex_grow: 1.,
            overflow: Overflow::hidden(),
        }
        ThemeBackgroundColor(tokens::SUBPANE_BODY_BG)
    }
}

/// Align this node's position and size with its parent.
pub fn fit_to_parent() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            top: Val::ZERO,
            bottom: Val::ZERO,
            left: Val::ZERO,
            right: Val::ZERO,
        }
    }
}
