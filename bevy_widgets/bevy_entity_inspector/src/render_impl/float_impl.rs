use bevy::prelude::*;

use bevy_field_forms::{input_field::InputField, validate_highlight::SimpleBorderHighlight};
use bevy_incomplete_bsn::entity_diff_tree::EntityDiffTree;

use crate::render::RenderContext;

pub fn float_render_impl(float: &f32, context: &RenderContext) -> EntityDiffTree {
    let mut tree = EntityDiffTree::new();

    let val = *float; //Clone the value to avoid borrowing issues

    tree.add_patch_fn(move |input: &mut InputField<f32>| {
        input.value = val;
    });

    tree.add_patch_fn(|node: &mut Node| {
        node.min_width = Val::Px(100.0);
        node.min_height = Val::Px(20.0);
        node.border = UiRect::all(Val::Px(1.0));
    });

    tree.add_patch_fn(|background: &mut BackgroundColor| {
        *background = BackgroundColor(Color::srgb(0.2, 0.2, 0.2));
    });

    tree.add_patch_fn(|border: &mut BorderColor| {
        *border = BorderColor(Color::srgb(0.3, 0.3, 0.3));
    });

    tree.add_patch_fn(|border_radius: &mut BorderRadius| {
        *border_radius = BorderRadius::all(Val::Px(5.0));
    });

    tree.add_patch_fn(|highlight: &mut SimpleBorderHighlight| {});

    tree
}
