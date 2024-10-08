use bevy::prelude::*;

use crate::NumericFieldValue;

/// Trigger for new numeric value changed by user input
#[derive(Event)]
pub struct NewValue<T: NumericFieldValue>(pub T);

/// Trigger with this event to change the numeric value
#[derive(Event)]
pub struct SetValue<T: NumericFieldValue>(pub T);
