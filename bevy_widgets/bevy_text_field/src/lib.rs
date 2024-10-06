//! This library provides a input text field widget for Bevy.
//!
//! Currently only single line text input is supported. (multiline text input is coming some time)

// These colors are taken from the Bevy website's text field in examples page
const BORDER_COLOR: Color = Color::srgb(56.0 / 255.0, 56.0 / 255.0, 56.0 / 255.0);
const HIGHLIGHT_COLOR: Color = Color::srgb(107.0 / 255.0, 107.0 / 255.0, 107.0 / 255.0);
const BACKGROUND_COLOR: Color = Color::srgb(43.0 / 255.0, 44.0 / 255.0, 47.0 / 255.0);

use bevy::prelude::*;
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
    text_right: Option<Entity>,
    cursor: Option<Entity>,
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
}

fn trigger_render_on_change(
    mut commands: Commands,
    q_fields: Query<Entity, Changed<LineTextField>>,
) {
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

        commands.entity(entity).add_child(canvas);
        commands.entity(canvas).add_child(text_field);
        commands.entity(entity).insert(Pickable {
            is_hoverable: true,
            should_block_lower: true
        });
        commands.entity(entity).insert(Interaction::default());

        let links = LineTextFieldLinks {
            canvas,
            text: text_field,
            text_right: None,
            cursor: None,
        };
        commands.entity(entity).insert(links);
    }
}

fn render_text_field(
    trigger: Trigger<RenderTextField>,
    mut commands: Commands,
    q_text_fields: Query<(&LineTextField, &LineTextFieldLinks, &Interaction)>,
    mut q_border_color: Query<&mut BorderColor>,
) {
    let entity = trigger.entity();
    let Ok((text_field, links, highlighted)) = q_text_fields.get(entity) else {
        return;
    };

    let border_color = if *highlighted == Interaction::Hovered || *highlighted == Interaction::Pressed {
        HIGHLIGHT_COLOR
    } else {
        BORDER_COLOR
    };

    let Ok(mut canvas_border_color) = q_border_color.get_mut(links.canvas) else {
        return;
    };

    *canvas_border_color = BorderColor(border_color);
}
