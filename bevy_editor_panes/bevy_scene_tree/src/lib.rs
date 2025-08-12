//! An interactive, collapsible tree view for hierarchical ECS data in Bevy.

use bevy::{app::Plugin, color::palettes::tailwind, prelude::*};
use bevy_editor_core::selection::EditorSelection;
use bevy_i_cant_believe_its_not_bsn::{Template, TemplateEntityCommandsExt, on, template};
use bevy_pane_layout::prelude::{PaneAppExt, PaneStructure};

/// Plugin for the editor scene tree pane.
pub struct SceneTreePlugin;

impl Plugin for SceneTreePlugin {
    fn build(&self, app: &mut App) {
        app.register_pane("Scene Tree", setup_pane)
            .add_systems(PostUpdate, update_scene_tree);
    }
}

/// Root UI node of the scene tree.
#[derive(Component)]
struct SceneTreeRoot;


fn setup_pane(pane: In<PaneStructure>, mut commands: Commands) {
    commands
        .entity(pane.content)
        .insert((
            SceneTreeRoot,
            Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                column_gap: Val::Px(2.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..Default::default()
            },
            BackgroundColor(tailwind::NEUTRAL_600.into()),
        ))
        .observe(
            |mut trigger: On<Pointer<Click>>, mut selection: ResMut<EditorSelection>| {
                selection.clear();
                trigger.propagate(false);
            },
        );
}

fn update_scene_tree(
    scene_trees: Query<Entity, With<SceneTreeRoot>>,
    scene_entities: Query<(Entity, &Name)>,
    selection: Res<EditorSelection>,
    mut commands: Commands,
) {
    for scene_tree in &scene_trees {
        let tree_rows: Template = scene_entities
            .iter()
            .flat_map(|(entity, name)| scene_tree_row_for_entity(entity, name, &selection, 0))
            .collect();

        commands.entity(scene_tree).build_children(tree_rows);
    }
}

fn scene_tree_row_for_entity(entity: Entity, name: &Name, selection: &EditorSelection, level: usize) -> Template {
    let selection_handler =
        move |mut trigger: On<Pointer<Click>>,
              keyboard_input: Res<ButtonInput<KeyCode>>,
              mut selection: ResMut<EditorSelection>| {
            if trigger.button != PointerButton::Primary {
                return;
            }

            trigger.propagate(false);
            let ctrl = keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
            if ctrl {
                selection.toggle(entity);
            } else {
                selection.set(entity);
            }
        };

    let indentation_px = level * 20;
    
    template! {
        {entity}: (
            Node {
                padding: UiRect::new(Val::Px(4.0 + indentation_px as f32), Val::Px(4.0), Val::Px(2.0), Val::Px(2.0)),
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            BorderRadius::all(Val::Px(4.0)),
            BackgroundColor(if selection.contains(entity) { tailwind::BLUE_600.into() } else { Color::NONE }),
        ) => [
            on(selection_handler);
            // Indentation spacer
            (
                Node {
                    width: Val::Px(16.0),
                    height: Val::Px(16.0),
                    margin: UiRect::right(Val::Px(4.0)),
                    ..default()
                },
            );
            // Entity name
            (
                Text(name.into()),
                TextFont::from_font_size(12.0),
                TextColor(if selection.contains(entity) { Color::WHITE } else { tailwind::NEUTRAL_200.into() }),
                Pickable::IGNORE,
            );
        ];
    }
}
