//! This library provides a input text field widget for Bevy.
//!
//! Currently only single line text input is supported. (multiline text input is coming some time)

// These colors are taken from the Bevy website's text field in examples page
const BORDER_COLOR: Color = Color::srgb(56.0 / 255.0, 56.0 / 255.0, 56.0 / 255.0);
const HIGHLIGHT_COLOR: Color = Color::srgb(107.0 / 255.0, 107.0 / 255.0, 107.0 / 255.0);
const BACKGROUND_COLOR: Color = Color::srgb(43.0 / 255.0, 44.0 / 255.0, 47.0 / 255.0);

use bevy::{input::keyboard::{Key, KeyboardInput}, prelude::*};
use bevy_focus::Focus;

pub struct LineTextFieldPlugin;

impl Plugin for LineTextFieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RenderTextField>();

        app.observe(render_text_field);
        app.observe(text_field_on_over);
        app.observe(text_field_on_out);
        app.observe(text_field_on_click);

        app.add_systems(PreUpdate, (spawn_render_text_field, trigger_render_on_change).chain());
        app.add_systems(Update, update_cursor);
        app.add_systems(Update, keyboard_input);
    }
}

#[derive(Component)]
pub struct Cursor {
    timer: Timer,
    visible: bool
}

impl Default for Cursor {
    fn default() -> Self {
        Self { 
            timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            visible: true
        }
    }
}

#[derive(Event)]
pub struct RenderTextField;

/// Single line text field
#[derive(Component, Default, Reflect)]
pub struct LineTextField {
    /// Text in the text field
    pub text: String,
    /// Cursor position
    pub cursor_position: Option<usize>,
}

#[derive(Component)]
struct LineTextFieldLinks {
    canvas: Entity,
    text: Entity,
    text_right: Entity,
    cursor: Entity,
}

impl LineTextField {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            cursor_position: None,
        }
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }
}

#[derive(Component)]
pub struct Highlighted;

fn text_field_on_over(
    over: Trigger<Pointer<Over>>,
    mut commands: Commands,
    q_text_fields: Query<&LineTextField>,
) {
    let entity = over.entity();
    info!("Over: {:?}", entity);

    // Observers attached to child of LineTextField, so we must get text field entity from parent componeny
    let Ok(_) = q_text_fields.get(entity) else {
        return;
    };

    commands.entity(entity).insert(Highlighted);
    commands.trigger_targets(RenderTextField, entity);
}

fn text_field_on_out(
    out: Trigger<Pointer<Out>>,
    mut commands: Commands,
    q_text_fields: Query<&LineTextField>,
) {
    let entity = out.entity();
    info!("Out: {:?}", entity);
    // Observers attached to child of LineTextField, so we must get text field entity from parent componeny
    let Ok(_) = q_text_fields.get(entity) else {
        return;
    };

    commands.entity(entity).remove::<Highlighted>();
    commands.trigger_targets(RenderTextField, entity);
}

fn text_field_on_click(
    click: Trigger<Pointer<Click>>,
    mut commands: Commands,
    mut q_text_fields: Query<(&mut LineTextField, Option<&Focus>)>,
) {
    info!("Click: {:?}", click.entity());
    let entity = click.entity();

    let Ok((mut text_field, focus)) = q_text_fields.get_mut(entity) else {
        return;
    };

    if focus.is_some() {
        commands.entity(entity).remove::<Focus>();
        text_field.cursor_position = None;
        return;
    }

    commands.entity(entity).insert(Focus);
    text_field.cursor_position = Some(text_field.text.len());
    commands.trigger_targets(RenderTextField, entity);
}

fn trigger_render_on_change(
    mut commands: Commands,
    q_fields: Query<Entity, Changed<LineTextField>>,
) {
    if q_fields.is_empty() {
        return;
    }
    info!("Changed: {:?}", q_fields.iter().collect::<Vec<_>>());
    commands.trigger_targets(RenderTextField, q_fields.iter().collect::<Vec<_>>());
}

fn spawn_render_text_field(
    mut commands: Commands,
    q_text_fields: Query<(Entity, &LineTextField), Added<LineTextField>>,
) {
    for (entity, text_field) in q_text_fields.iter() {

        let canvas = commands.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                padding: UiRect::all(Val::Px(5.0)),
                ..Default::default()
            },
            background_color: BackgroundColor(BACKGROUND_COLOR.clone()),
            border_color: BorderColor(BORDER_COLOR.clone()),
            border_radius: BorderRadius::all(Val::Px(5.0)),
            ..Default::default()
        })
        .id();

        let text_field = commands.spawn(TextBundle::from_section(
            text_field.text.clone(),
            TextStyle::default(),
        )).id();

        let text_field_right = commands.spawn(TextBundle::from_section("", TextStyle::default())).id();
        let cursor = commands.spawn(NodeBundle::default()).id();

        commands.entity(entity).add_child(canvas);
        commands.entity(canvas).add_child(text_field);
        commands.entity(canvas).add_child(cursor);
        commands.entity(canvas).add_child(text_field_right);
        commands.entity(entity).insert(Pickable {
            is_hoverable: true,
            should_block_lower: true
        });
        commands.entity(entity).insert(Interaction::default());

        let links = LineTextFieldLinks {
            canvas,
            text: text_field,
            text_right: text_field_right,
            cursor: cursor,
        };
        commands.entity(entity).insert(links);
    }
}

fn render_text_field(
    mut trigger: Trigger<RenderTextField>,
    mut commands: Commands,
    q_text_fields: Query<(&LineTextField, &LineTextFieldLinks, &Interaction)>,
    mut q_border_color: Query<&mut BorderColor>,
) {
    let entity = trigger.entity();
    trigger.propagate(false);

    let Ok((text_field, links, highlighted)) = q_text_fields.get(entity) else {
        return;
    };

    info!("Render: {:?}", trigger.entity());

    let border_color = if *highlighted == Interaction::Hovered || *highlighted == Interaction::Pressed {
        HIGHLIGHT_COLOR
    } else {
        BORDER_COLOR
    };

    let Ok(mut canvas_border_color) = q_border_color.get_mut(links.canvas) else {
        return;
    };

    *canvas_border_color = BorderColor(border_color);

    if let Some(cursor) = text_field.cursor_position {
        let left_text = text_field.text[..cursor].to_string();
        let right_text = text_field.text[cursor..].to_string();
        commands.entity(links.text).insert(TextBundle::from_section(left_text, TextStyle::default()));
        commands.entity(links.text_right).insert(TextBundle::from_section(right_text, TextStyle::default()));
        commands.entity(links.cursor).insert(NodeBundle {
            style: Style {
                height: Val::Percent(100.0),
                width: Val::Px(2.0),
                ..default()
            },
            background_color: BackgroundColor(Color::WHITE),
            ..default()
        }).insert(Cursor::default());
    } else {
        commands.entity(links.text).insert(TextBundle::from_section(text_field.text.clone(), TextStyle::default()));
        commands.entity(links.cursor).insert(NodeBundle::default()).remove::<Cursor>();
        commands.entity(links.text_right).insert(TextBundle::from_section("", TextStyle::default()));
    }
}

fn update_cursor(
    time: Res<Time>,
    mut q_cursors: Query<(&mut Cursor, &mut Visibility)>
) {
    for (mut cursor, mut visibility) in q_cursors.iter_mut() {
        if cursor.timer.tick(time.delta()).just_finished() {
            cursor.visible = !cursor.visible;
            if cursor.visible {
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}


fn keyboard_input(
    mut commands: Commands,
    mut q_text_fields: Query<(Entity, &mut LineTextField), With<Focus>>,
    mut events: EventReader<KeyboardInput>,
) {
    let Ok((entity, mut text_field)) = q_text_fields.get_single_mut() else {
        return;
    };

    let Some(mut current_cursor) = text_field.cursor_position else {
        return;
    };

    let mut need_render = false;

    for event in events.read() {
        if !event.state.is_pressed() {
            continue;
        }

        match &event.logical_key {
            Key::Space => {
                need_render = true;
                text_field.text.insert(current_cursor, ' ');
                text_field.cursor_position = Some(current_cursor + 1);
            }
            Key::Backspace => {
                need_render = true;
                if current_cursor > 0 {
                    text_field.text.remove(current_cursor - 1);
                    text_field.cursor_position = Some(current_cursor - 1);
                }
            }
            Key::Character(c) => {
                need_render = true;
                for c in c.chars() {
                    text_field.text.insert(current_cursor, c);
                    current_cursor += 1;
                }
                text_field.cursor_position = Some(current_cursor);
            },
            Key::ArrowLeft => {
                if current_cursor > 0 {
                    text_field.cursor_position = Some(current_cursor - 1);
                    need_render = true;
                }
            }
            Key::ArrowRight => {
                if current_cursor < text_field.text.len() {
                    text_field.cursor_position = Some(current_cursor + 1);
                    need_render = true;
                }
            }
            _ => {}
        }
    }

    if need_render {
        commands.trigger_targets(RenderTextField, entity);
    }

}