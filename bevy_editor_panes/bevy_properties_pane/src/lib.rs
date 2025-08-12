//! An interactive, reflection-based inspector for Bevy ECS data in running applications.
//!
//! Data can be viewed and modified in real-time, with changes being reflected in the application.

use bevy::{
    feathers::theme::ThemedText,
    prelude::*,
    reflect::*,
    scene2::{CommandsSpawnScene, Scene, SceneList, bsn},
};
use bevy_editor_styles::Theme;
use bevy_editor_core::{prelude::*, selection::common_conditions::primary_selection_changed};
use bevy_pane_layout::prelude::*;

/// Plugin for the editor properties pane.
pub struct PropertiesPanePlugin;

impl Plugin for PropertiesPanePlugin {
    fn build(&self, app: &mut App) {
        app.register_pane("Properties", setup_pane)
            .add_systems(
                Update,
                (
                    update_properties_pane.run_if(
                        primary_selection_changed.or(any_match_filter::<Added<PropertiesPaneBody>>),
                    ),
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
        .insert(ChildOf(pane.root));
}

fn update_properties_pane(
    pane_bodies: Query<Entity, With<PropertiesPaneBody>>,
    selection: Res<EditorSelection>,
    theme: Res<Theme>,
    world: &World,
    mut commands: Commands,
) {
    for pane_body in &pane_bodies {
        commands.entity(pane_body).despawn_children();
        commands
            .spawn_scene(properties_pane(&selection, &theme, world))
            .insert(ChildOf(pane_body));
    }
}

fn properties_pane(selection: &EditorSelection, theme: &Theme, world: &World) -> impl Scene {
    match selection.primary() {
        Some(selection) => bsn! {
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(6.0)
            } [
            {component_list(selection, theme, world)}
        ]}
        .boxed_scene(),
        None => bsn! {
            Node {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(24.0))
            } [
                Text("Select an entity to inspect")
                TextFont::from_font_size(14.0)
                TextColor(Color::srgb(0.514, 0.514, 0.522)),
            ]
        }
        .boxed_scene(),
    }
}


fn component_list(entity: Entity, theme: &Theme, world: &World) -> impl SceneList {
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
                    margin: UiRect::bottom(Val::Px(6.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::all(Val::Px(0.0))
                }
                // CSS: #2A2A2E - Component background  
                BackgroundColor(Color::srgb(0.165, 0.165, 0.180))
                // CSS: #414142 - Border color  
                BorderColor::all(Color::srgb(0.255, 0.255, 0.259))
                BorderRadius::all(Val::Px(5.0))
                [
                    // Component header - CSS styling
                    Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(8.0)),
                        height: Val::Px(26.0)
                    }
                    // CSS: #36373B - Header background
                    BackgroundColor(Color::srgb(0.212, 0.216, 0.231))
                    BorderRadius::top(Val::Px(5.0))
                    [
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(5.0)
                        } [
                            Text("▼")
                            TextFont::from_font_size(12.0)
                            // CSS: #C4C4C4 - Chevron color
                            TextColor(Color::srgb(0.769, 0.769, 0.769)),
                            
                            Text({format!("{name}")})
                            TextFont::from_font_size(12.0)
                            // CSS: #DCDCDC - Component name
                            TextColor(Color::srgb(0.863, 0.863, 0.863)),
                        ],
                        
                        Text("⋯")
                        TextFont::from_font_size(12.0)
                        // CSS: #C4C4C4 - Menu dots
                        TextColor(Color::srgb(0.769, 0.769, 0.769)),
                    ],
                    // Component fields
                    ({ match reflect {
                        Some(reflect) => component(type_info, reflect, theme).boxed_scene(),
                        None => bsn! {
                            Node {
                                flex_direction: FlexDirection::Row,
                                padding: UiRect::all(Val::Px(8.0))
                            } [
                                Text("<reflection unavailable>")
                                TextFont::from_font_size(11.0)
                                TextColor(Color::srgb(0.514, 0.514, 0.522)),
                            ]
                        }.boxed_scene(),
                    }}),
                ]
            }
        })
        .collect::<Vec<_>>()
}

fn component(type_info: Option<&TypeInfo>, reflect: &dyn Reflect, theme: &Theme) -> impl Scene {
    match type_info {
        Some(TypeInfo::Struct(info)) => reflected_struct(info, reflect, theme).boxed_scene(),
        Some(TypeInfo::TupleStruct(info)) => reflected_tuple_struct(info, theme).boxed_scene(),
        Some(TypeInfo::Enum(info)) => reflected_enum(info, theme).boxed_scene(),
        _ => bsn! {}.boxed_scene(),
    }
}
fn reflected_struct(struct_info: &StructInfo, reflect: &dyn Reflect, theme: &Theme) -> impl Scene {
    let fields = struct_info
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let field_reflect = reflect
                .reflect_ref()
                .as_struct()
                .ok()
                .and_then(|s| s.field_at(i));

            let field_name = field.name();
            
            let value_string = field_reflect
                .map(|v| format!("{v:?}"))
                .unwrap_or_else(|| "<unavailable>".to_string());

            bsn! {
                Node {
                    flex_direction: FlexDirection::Row,
                    margin: UiRect::vertical(Val::Px(2.0)),
                    padding: UiRect::all(Val::Px(5.0)),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    min_height: Val::Px(22.0)
                }
                // CSS: #36373B - Field background
                BackgroundColor(Color::srgb(0.212, 0.216, 0.231))
                BorderRadius::all(Val::Px(3.0))
                [
                    Text(field_name)
                    TextFont::from_font_size(12.0)
                    // CSS: #DADADA - Field labels
                    TextColor(Color::srgb(0.855, 0.855, 0.855)),
                    
                    Text({value_string.clone()})
                    TextFont::from_font_size(12.0)
                    // CSS: #C2C2C2 - Field values
                    TextColor(Color::srgb(0.761, 0.761, 0.761)),
                ]
            }
        })
        .collect::<Vec<_>>();

    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(7.0)),
            row_gap: Val::Px(4.0)
        } [ {fields} ]
    }
}


fn reflected_tuple_struct(tuple_struct_info: &TupleStructInfo, theme: &Theme) -> impl Scene {
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
            flex_direction: FlexDirection::Column
        } [ {fields} ]
    }
}

fn reflected_enum(enum_info: &EnumInfo, theme: &Theme) -> impl Scene {
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
