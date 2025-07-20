//! An interactive, reflection-based inspector for Bevy ECS data in running applications.
//!
//! Data can be viewed and modified in real-time, with changes being reflected in the application.

use bevy::{
    feathers::theme::ThemedText,
    prelude::*,
    reflect::*,
    scene2::{CommandsSpawnScene, Scene, SceneList, bsn},
};
use bevy_editor_core::{selection::SelectedEntity, utils::IntoBoxedScene};
use bevy_pane_layout::prelude::*;

/// Plugin for the editor properties pane.
pub struct PropertiesPanePlugin;

impl Plugin for PropertiesPanePlugin {
    fn build(&self, app: &mut App) {
        app.register_pane("Properties", setup_pane).add_systems(
            Update,
            update_properties_pane.run_if(
                resource_changed::<SelectedEntity>
                    .or(any_match_filter::<Added<PropertiesPaneBody>>),
            ),
        );
    }
}

/// Root UI node of the properties pane.
#[derive(Component, Default, Clone)]
struct PropertiesPaneBody;

fn setup_pane(pane: In<PaneStructure>, mut commands: Commands) {
    // Remove the existing structure
    commands.entity(pane.area).despawn();

    commands
        .spawn_scene(bsn! {
            :editor_pane [
                :editor_pane_header [
                    (Text("Properties") ThemedText),
                ],
                :editor_pane_body
                PropertiesPaneBody
            ]
        })
        .insert(Node::default())
        .insert(ChildOf(pane.root));
}

fn update_properties_pane(
    pane_bodies: Query<Entity, With<PropertiesPaneBody>>,
    selected_entity: Res<SelectedEntity>,
    world: &World,
    mut commands: Commands,
) {
    for pane_body in &pane_bodies {
        commands.entity(pane_body).despawn_children();
        commands
            .spawn_scene(properties_pane(&selected_entity, world))
            .insert(Node::default())
            .insert(ChildOf(pane_body));
    }
}

fn properties_pane(selected_entity: &SelectedEntity, world: &World) -> impl Scene {
    match selected_entity.0 {
        Some(selected_entity) => bsn! {Node { flex_direction: FlexDirection::Column } [
            {component_list(selected_entity, world)}
        ]}
        .boxed_scene(),
        None => bsn! {
            (Text("Select an entity to inspect") ThemedText)
        }
        .boxed_scene(),
    }
}

fn component_list(entity: Entity, world: &World) -> impl SceneList {
    let type_registry = world.resource::<AppTypeRegistry>().read();
    world
        .inspect_entity(entity)
        .unwrap()
        .map(|component_info| {
            let type_info = component_info
                .type_id()
                .and_then(|type_id| type_registry.get_type_info(type_id));
            let name = type_info.map_or_else(
                || "<unknown>".to_string(),
                |type_info| type_info.type_path_table().short_path().to_string(),
            );

            // Get the reflected component value from the world
            let reflect: Option<&dyn Reflect> = component_info.type_id().and_then(|type_id| {
                let registration = type_registry.get(type_id)?;
                let reflect_component = registration.data::<ReflectComponent>()?;
                let entity_ref = world.get_entity(entity);
                reflect_component.reflect(entity_ref.unwrap())
            });

            bsn! {
                Node {
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::all(Val::Px(4.0)),
                } [
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                    } [
                        TextFont::from_font_size(14.0)
                        Text({format!("{name}")})
                        TextColor(Color::WHITE)
                    ],
                    // Component fields
                    ({ match reflect {
                        Some(reflect) => component(type_info, reflect).boxed_scene(),
                        None => bsn! {
                            Node {
                                flex_direction: FlexDirection::Row,
                            } [
                                Text("<unavailable>")
                                TextFont::from_font_size(10.0)
                                TextColor(Color::srgb(1.0, 0.0, 0.0))
                            ]
                        }.boxed_scene(),
                    }})
                ]
            }
        })
        .collect::<Vec<_>>()
}

fn component(type_info: Option<&TypeInfo>, reflect: &dyn Reflect) -> impl Scene {
    match type_info {
        Some(TypeInfo::Struct(info)) => reflected_struct(info, reflect).boxed_scene(),
        Some(TypeInfo::TupleStruct(info)) => reflected_tuple_struct(info).boxed_scene(),
        Some(TypeInfo::Enum(info)) => reflected_enum(info).boxed_scene(),
        _ => bsn! {}.boxed_scene(),
    }
}
fn reflected_struct(struct_info: &StructInfo, reflect: &dyn Reflect) -> impl Scene {
    let fields = struct_info
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let valuee = reflect
                .reflect_ref()
                .as_struct()
                .map(|s| s.field_at(i))
                .map(|v| format!("{v:?}"))
                .unwrap_or("<unavailable>".to_string());

            let field_name = field.name();
            bsn! {
                Node {
                    flex_direction: FlexDirection::Row,
                    margin: UiRect::vertical(Val::Px(2.0)),
                } [
                    (
                        Text(field_name)
                        TextFont::from_font_size(12.0)
                        TextColor(Color::srgb(0.8, 0.8, 0.8))
                    ),
                    (
                        // Value (use reflection to get value as string)
                        Text({valuee.clone()})
                        TextFont::from_font_size(10.0)
                        TextColor(Color::WHITE)
                    ),
                ]
            }
        })
        .collect::<Vec<_>>();

    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
        } [ {fields} ]
    }
}

fn reflected_tuple_struct(tuple_struct_info: &TupleStructInfo) -> impl Scene {
    let fields = tuple_struct_info
        .iter()
        .map(|_field| {
            bsn! {
                Text("TODO")
                TextFont::from_font_size(10.0)
            }
        })
        .collect::<Vec<_>>();

    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
        } [ {fields} ]
    }
}

fn reflected_enum(enum_info: &EnumInfo) -> impl Scene {
    let variants = enum_info
        .iter()
        .map(|variant| {
            let name = variant.name();
            bsn! {
                Text(name)
                TextFont::from_font_size(10.0)
            }
        })
        .collect::<Vec<_>>();

    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
        } [ {variants} ]
    }
}
