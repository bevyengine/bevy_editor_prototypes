use std::sync::Arc;

use bevy::prelude::*;

use bevy_field_forms::{
    drag_input::DragInput,
    input_field::{InputField, ValueChanged},
    validate_highlight::SimpleBorderHighlight,
};
use bevy_incomplete_bsn::entity_diff_tree::EntityDiffTree;

use crate::render::{ChangeComponentField, RenderContext};

pub fn float_render_impl(float: &f32, path: String, context: &RenderContext) -> EntityDiffTree {
    let mut tree = EntityDiffTree::new();

    let val = *float; //Clone the value to avoid borrowing issues

    tree.add_patch_fn(move |input: &mut InputField<f32>| {
        input.value = val;
    });

    tree.add_patch_fn(|drag: &mut DragInput<f32>| {});

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

    tree.add_observer_patch(
        move |trigger: Trigger<ValueChanged<f32>>, mut commands: Commands| {
            info!(
                "Trigger reflect change with path: {} and value: {}",
                path, trigger.0
            );
            let entity = trigger.entity();

            commands.trigger_targets(
                ChangeComponentField {
                    value: Arc::new(trigger.0),
                    path: path.clone(),
                    direct_cange: Some(Arc::new(|dst, src| {
                        let dst_f32 = dst.try_downcast_mut::<f32>().unwrap();
                        let src_f32 = src.try_downcast_ref::<f32>().unwrap();
                        *dst_f32 = *src_f32;
                    })),
                },
                entity,
            );
        },
    );

    tree
}
