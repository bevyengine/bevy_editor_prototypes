//! An interactive, reflection-based inspector for Bevy ECS data in running applications.
//!
//! Data can be viewed and modified in real-time, with changes being reflected in the application.

use bevy::{color::palettes::tailwind, prelude::*, reflect::*};
use bevy_editor_core::SelectedEntity;
use bevy_i_cant_believe_its_not_bsn::{template, Template, TemplateEntityCommandsExt};
use bevy_pane_layout::prelude::{PaneAppExt, PaneStructure};

/// Plugin for the editor properties pane.
pub struct PropertiesPanePlugin;

impl Plugin for PropertiesPanePlugin {
    fn build(&self, app: &mut App) {
        app.register_pane("Properties", setup_pane)
            .add_systems(PostUpdate, update_properties_pane);
    }
}

/// Root UI node of the properties pane.
#[derive(Component)]
struct PropertiesPaneRoot;

fn setup_pane(pane: In<PaneStructure>, mut commands: Commands) {
    commands.entity(pane.content).insert((
        PropertiesPaneRoot,
        Node {
            flex_direction: FlexDirection::Column,
            flex_grow: 1.0,
            column_gap: Val::Px(4.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..Default::default()
        },
        BackgroundColor(tailwind::NEUTRAL_600.into()),
    ));
}

fn update_properties_pane(
    panes: Query<Entity, With<PropertiesPaneRoot>>,
    selected_entity: Res<SelectedEntity>,
    world: &World,
    mut commands: Commands,
) {
    for pane in &panes {
        commands
            .entity(pane)
            .build_children(properties_pane(&selected_entity, world));
    }
}

fn properties_pane(selected_entity: &SelectedEntity, world: &World) -> Template {
    match selected_entity.0 {
        Some(selected_entity) => component_list(selected_entity, world),
        None => template! {(
            Text("Select an entity to inspect".into()),
            TextFont::from_font_size(14.0),
        );},
    }
}

fn component_list(entity: Entity, world: &World) -> Template {
    let type_registry = world.resource::<AppTypeRegistry>().read();
    world
        .inspect_entity(entity)
        .flat_map(|component_info| {
            let (_, name) = component_info.name().rsplit_once("::").unwrap();
            let type_info = component_info
                .type_id()
                .and_then(|type_id| type_registry.get_type_info(type_id));

            template! {
                Node {
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                } => [
                    (
                        Text(name.into()),
                        TextFont::from_font_size(12.0),
                    );
                    @{ component(type_info) };
                ];
            }
        })
        .collect()
}

fn component(type_info: Option<&TypeInfo>) -> Template {
    match type_info {
        Some(type_info) => reflected_component(type_info),
        None => template! {(
            Text("Reflect not implemented".into()),
            TextFont::from_font_size(10.0),
            TextColor(tailwind::NEUTRAL_300.into()),
        );},
    }
}

fn reflected_component(type_info: &TypeInfo) -> Template {
    match type_info {
        TypeInfo::Struct(struct_info) => reflected_struct(struct_info),
        TypeInfo::TupleStruct(tuple_struct_info) => reflected_tuple_struct(tuple_struct_info),
        TypeInfo::Tuple(_tuple_info) => todo!(),
        TypeInfo::List(_list_info) => todo!(),
        TypeInfo::Array(_array_info) => todo!(),
        TypeInfo::Map(_map_info) => todo!(),
        TypeInfo::Set(_set_info) => todo!(),
        TypeInfo::Enum(enum_info) => reflected_enum(enum_info),
        TypeInfo::Opaque(_opaque_info) => todo!(),
    }
}

fn reflected_struct(struct_info: &StructInfo) -> Template {
    let fields = struct_info
        .iter()
        .flat_map(|field| {
            template! {(
                Text(field.name().into()),
                TextFont::from_font_size(10.0),
            );}
        })
        .collect::<Template>();

    template! {
        Node {
            flex_direction: FlexDirection::Column,
            ..Default::default()
        } => [ @{ fields }; ];
    }
}

fn reflected_tuple_struct(tuple_struct_info: &TupleStructInfo) -> Template {
    let fields = tuple_struct_info
        .iter()
        .flat_map(|_field| {
            template! {(
                Text("TODO".into()),
                TextFont::from_font_size(10.0),
            );}
        })
        .collect::<Template>();

    template! {
        Node {
            flex_direction: FlexDirection::Column,
            ..Default::default()
        } => [ @{ fields }; ];
    }
}

fn reflected_enum(enum_info: &EnumInfo) -> Template {
    let variants = enum_info
        .iter()
        .flat_map(|variant| {
            template! {(
                Text(variant.name().into()),
                TextFont::from_font_size(10.0),
            );}
        })
        .collect::<Template>();

    template! {
        Node {
            flex_direction: FlexDirection::Column,
            ..Default::default()
        } => [ @{ variants }; ];
    }
}
