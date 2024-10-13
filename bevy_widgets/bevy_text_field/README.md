# bevy_text_field

A flexible and feature-rich text field widget for the Bevy game engine.

## Overview

`bevy_text_field` provides a customizable text input widget for Bevy applications, offering a robust solution for single-line text editing with features like cursor positioning, text selection, and clipboard operations.

## Features

- Single-line text editing with real-time updates
- Precise cursor positioning and text selection
- Clipboard operations (copy, cut, paste, select all via Ctrl+C, X, V, A)
- Horizontal scrolling for text exceeding field width
- Customizable character set restrictions

## Installation

Add `bevy_text_field` to your `Cargo.toml`:

```toml
[dependencies]
bevy_text_field = "0.1.0"
```

## Usage

Here's a basic example of how to use `bevy_text_field` in your Bevy app:

```rust
use bevy::prelude::*;
use bevy_text_field::{LineTextField, LineTextFieldPlugin, TextChanged};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LineTextFieldPlugin)
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
        LineTextField::new("Initial text"),
    )).observe(|trigger: Trigger<TextChanged>| {
        let text = trigger.event().0;
        println!("Text changed: {}", text);
    });
}
```

### Updating Text Programmatically

You can update the text field's content programmatically:

```rust
fn update_text_system(mut query: Query<&mut LineTextField>) {
    if let Ok(mut text_field) = query.get_single_mut() {
        text_field.set_text("Updated text");
    }
}
```

### Restricting Allowed Characters

Limit input to specific characters:

```rust
fn setup_restricted_field(mut commands: Commands) {
    let mut text_field = LineTextField::new("123");
    text_field.set_allowed_chars(['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']);
    
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Px(100.0),
                height: Val::Px(30.0),
                ..Default::default()
            },
            ..Default::default()
        },
        text_field,
    ));
}
```

## How It Works

Here's a breakdown of its core components and systems:

1. **Entity Structure**:
   - The main `LineTextField` entity contains the text data and input state.
   - Child entities handle rendering for the text, cursor, and selection highlight.

2. **Rendering**:
   - The text is split into two parts: before and after the cursor.
   - A cursor entity is positioned between these text parts.
   - For selected text, an additional entity renders the highlight.

3. **Scrolling**:
   - When text exceeds the field width, a scrolling system adjusts the visible portion.

4. **Update Cycle**:
   - Changes to the `LineTextField` component trigger re-rendering.
   - The `TextChanged` event is emitted for external systems to react to changes.

5. **Clipboard Integration**:
   - Clipboard operations are handled through the `arboard` crate.
   - Copy, cut, and paste operations are bound to standard keyboard shortcuts.

## Limitations

- Single-line editing only (multi-line support planned for future releases)
- Limited to basic text input (no rich text formatting)
- Default character set is Latin-1 (customizable)
- Cursor positioning may be imprecise with variable-width fonts

## Roadmap

- [ ] Multi-line text editing support
- [ ] Rich text formatting options
- [ ] Improved cursor positioning for variable-width fonts
- [ ] Custom key bindings for clipboard operations