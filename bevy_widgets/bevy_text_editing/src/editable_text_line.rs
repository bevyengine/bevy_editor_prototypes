//! This file contains the [`EditableTextLine`] component which allow create editable text by keyboard and mouse

use bevy::{prelude::*, text::{cosmic_text::Buffer, TextLayoutInfo}, ui::experimental::GhostNode, utils::HashSet};
use bevy_focus::{FocusPlugin, SetFocus};

use crate::cursor::{Cursor, CursorPlugin};


pub struct EditableTextLinePlugin;

impl Plugin for EditableTextLinePlugin {
     fn build(&self, app: &mut App) {
          // Check that our required plugins are loaded.
         if !app.is_plugin_added::<CursorPlugin>() {
             app.add_plugins(CursorPlugin);
         }
         if !app.is_plugin_added::<FocusPlugin>() {
             app.add_plugins(FocusPlugin);
         }

         app.add_event::<SetText>();
         app.add_event::<TextChanged>();

         app.add_systems(PreUpdate, spawn_system);
         app.add_observer(set_text_trigger);
         app.add_observer(on_click);
     }
    
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
#[require(Node)]
pub struct EditableTextLine {
     /// Text content
     pub text: String,
     /// Cursor position
     pub cursor_position: Option<usize>,
     /// Selection start
     pub selection_start: Option<usize>,
     /// Controlled widgets do not update their state by themselves,
     /// while uncontrolled widgets can edit their own state.
     pub controlled_widget: bool
}

impl EditableTextLine {
     /// Create new editable text line
     pub fn new(text: impl Into<String>) -> Self {
          Self {
               text: text.into(),
               ..default()
          }
     }

     /// Create controlled editable text line
     pub fn controlled(text: impl Into<String>) -> Self {
          Self {
               text: text.into(),
               controlled_widget: true,
               ..default()
          }
     }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct EditableTextInner {
     fake_cursor_text: Entity,
     cursor : Entity,
     text: Entity,
     canvas: Entity,
}

#[derive(Event)]
pub struct SetText(pub String);

#[derive(Event)]
pub struct TextChanged(pub String);

fn spawn_system(
    mut commands: Commands,
    q_texts: Query<(Entity, &EditableTextLine), Without<EditableTextInner>>
) {
     for (e, text) in q_texts.iter() {
          let cursor = commands.spawn((
               Node {
                    width: Val::Px(2.0),
                    height: Val::Percent(100.0),
                    ..default()
               },
               Visibility::Hidden
          )).id();

          let fake_cursor_text = commands.spawn((
               Text::new("".to_string()),
               TextColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
               Node {
                    ..default()
               }
          )).id();

          let cursor_canvas = commands.spawn(
               Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
               }
          ).id();

          commands.entity(cursor_canvas).add_child(fake_cursor_text);
          commands.entity(cursor_canvas).add_child(cursor);


          let text = commands.spawn((
               Text::new(text.text.clone()),
               Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    ..default()
               }
          )).id();

          let canvas = commands.spawn(
               Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
               }
          ).id();

          commands.entity(canvas).add_child(text);
          commands.entity(canvas).add_child(cursor_canvas);

          commands.entity(e).insert(EditableTextInner {
               fake_cursor_text,
               cursor,
               text,
               canvas,
          }).add_child(canvas);
     }
}

fn set_text_trigger(
     trigger: Trigger<SetText>,
     mut q_texts: Query<&mut Text, With<EditableTextLine>>,
) {
     let entity = trigger.entity();
     let Ok(mut text) = q_texts.get_mut(entity) else {
         return;
     };

     text.0 = trigger.0.clone();
     info!("Set text for {} to {}", entity, trigger.0);
}


fn on_click(
    click: Trigger<Pointer<Click>>,
    mut commands: Commands,
    mut q_editable_texts: Query<(&mut EditableTextLine, &mut EditableTextInner)>,
    q_texts: Query<(&ComputedNode, &GlobalTransform)>
) {
     let entity = click.entity();
     let Ok((mut text_line, mut inner)) = q_editable_texts.get_mut(entity) else {
         return;
     };

     let Ok((node, global_transform)) = q_texts.get(inner.text) else {
         return;
     };

     let click_pos = click.pointer_location.position;
 
     let self_pos = global_transform.translation();

     let self_size = node.size();

     let dx_relative = (click_pos.x - self_pos.x) / self_size.x + 0.5;

     let mut cursor_pos = (text_line.text.chars().count() as f32 * dx_relative).round() as usize;
     let cursor_byte_pos;
     if cursor_pos < text_line.text.chars().count() {
          cursor_byte_pos = text_line.text.char_indices().nth(cursor_pos).unwrap().0;
     } else {
          cursor_pos = text_line.text.chars().count();
          cursor_byte_pos = text_line.text.len();
     }

     text_line.cursor_position = Some(cursor_pos);

     commands.entity(inner.cursor).insert((
          Cursor::default(), 
          Visibility::Visible, 
          BackgroundColor(Color::srgb(1.0, 1.0, 1.0))
     ));

     commands.entity(inner.fake_cursor_text).insert(Text::new(text_line.text[0..cursor_byte_pos].to_string()));

     commands.trigger_targets(SetFocus, entity);
}