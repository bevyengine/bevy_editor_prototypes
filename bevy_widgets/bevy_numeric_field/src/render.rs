use bevy::prelude::*;
use bevy_text_field::{LineTextField, LineTextFieldLinks};

use crate::{InnerNumericField, NumericFieldValue};

pub(crate) fn set_borders<T: NumericFieldValue>(
    mut q_styles: Query<&mut BorderColor>,
    q_fields: Query<(&LineTextFieldLinks, &InnerNumericField<T>)>,
) {
    for (links, inner) in q_fields.iter() {
        let Ok(mut canvas_border_color) = q_styles.get_mut(links.canvas) else {
            continue;
        };

        if inner.failed_convert {
            canvas_border_color.0 = Color::srgb(1.0, 0.0, 0.0);
        } else {
        }
    }
}
