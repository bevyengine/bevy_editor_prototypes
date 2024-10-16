# bevy_numeric_field

A flexible and feature-rich numeric input field widget for the Bevy game engine.

## Overview

`bevy_numeric_field` provides a customizable numeric input widget for Bevy applications, offering a robust solution for inputting and manipulating numeric values with support for various numeric types, including integers and floating-point numbers.

## Features

- Support for multiple numeric types (u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64)
- Mouse cursor positioning and text selection
- Clipboard operations (copy, cut, paste, select all via Ctrl+C, X, V, A)
- Drag functionality for incrementing/decrementing values
- Customizable min/max value constraints
- Efficient rendering and update systems

## Installation

Add `bevy_numeric_field` to your `Cargo.toml`:

```toml
[dependencies]
bevy_numeric_field = "0.1.0"
```

## Usage

Here's a basic example of how to use `bevy_numeric_field` in your Bevy app:

```rust
use bevy::prelude::*;
use bevy_numeric_field::{NumericField, NumericFieldPlugin, NewValue};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultNumericFieldPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Px(200.0),
                height: Val::Px(30.0),
                ..Default::default()
            },
            ..Default::default()
        },
        NumericField::<f32>::new(0.0),
    )).observe(|trigger: Trigger<NewValue<f32>>| {
        let value = trigger.event().0;
        println!("Value changed: {}", value);
    });
}
```

### Setting Constraints

You can set minimum and maximum values for the numeric field:

```rust
fn setup_constrained_field(mut commands: Commands) {
    let mut numeric_field = NumericField::<i32>::new(0);
    numeric_field.min = Some(-100);
    numeric_field.max = Some(100);
    
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Px(100.0),
                height: Val::Px(30.0),
                ..Default::default()
            },
            ..Default::default()
        },
        numeric_field,
    ));
}
```

## How It Works

`bevy_numeric_field` builds upon the `bevy_text_field` crate to create a specialized input widget for numeric values. Here's a breakdown of its core components and systems:

1. **Input Handling**:
   - Text input is parsed and validated against the specified numeric type.
   - Drag operations allow for incremental value changes.

2. **Constraints**:
   - Optional minimum and maximum values can be set to restrict input.

3. **Events**:
   - The `NewValue` event is emitted when the numeric value changes.
   - The `SetValue` event can be used to programmatically update the field's value.

4. **Rendering**:
   - The widget leverages `bevy_text_field` for text rendering and cursor management.
   - Additional visual feedback is provided for invalid inputs.