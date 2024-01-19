mod example_scene;

use bevy::{
    ecs::{archetype::Archetypes, component::Components, entity::Entities},
    prelude::*,
    reflect::{TypeInfo, TypeRegistry},
};
use bevy_egui::{
    egui::{self, Align2, Sense, Ui},
    EguiContexts, EguiPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_systems(Update, editor)
        .add_systems(Startup, example_scene::setup_example_scene)
        .run();
}

fn editor(
    mut contexts: EguiContexts,
    scene_entities: Query<(Entity, &Name)>,
    mut selected_entity: Local<Option<Entity>>,
    type_registry: Res<AppTypeRegistry>,
    entities: &Entities,
    archetypes: &Archetypes,
    components: &Components,
) {
    if let Some(currently_selected_entity) = *selected_entity {
        if !entities.contains(currently_selected_entity) {
            *selected_entity = None;
        }
    }

    egui::Window::new("Scene Tree")
        .anchor(Align2::LEFT_TOP, [0.0; 2])
        .resizable(false)
        .collapsible(false)
        .show(contexts.ctx_mut(), |ui| {
            scene_tree(ui, scene_entities, &mut selected_entity);
        });

    egui::Window::new("Entity Inspector")
        .anchor(Align2::RIGHT_TOP, [0.0; 2])
        .resizable(false)
        .collapsible(false)
        .show(contexts.ctx_mut(), |ui| {
            entity_inspector(
                ui,
                &selected_entity,
                &type_registry.read(),
                entities,
                archetypes,
                components,
            );
        });
}

fn scene_tree(
    ui: &mut Ui,
    scene_entities: Query<(Entity, &Name)>,
    selected_entity: &mut Option<Entity>,
) {
    for (entity, name) in &scene_entities {
        if ui
            .add(egui::Label::new(name.as_str()).sense(Sense::click()))
            .clicked()
        {
            *selected_entity = Some(entity);
        }
    }
}

fn entity_inspector(
    ui: &mut Ui,
    selected_entity: &Option<Entity>,
    type_registry: &TypeRegistry,
    entities: &Entities,
    archetypes: &Archetypes,
    components: &Components,
) {
    if let Some(entity) = selected_entity {
        for (type_info, name) in
            get_reflected_component_data(*entity, type_registry, entities, archetypes, components)
        {
            match type_info {
                TypeInfo::Struct(_) => ui.label(format!("{name}: Struct")),
                TypeInfo::TupleStruct(_) => ui.label(format!("{name}: TupleStruct")),
                TypeInfo::Tuple(_) => ui.label(format!("{name}: Tuple")),
                TypeInfo::List(_) => ui.label(format!("{name}: List")),
                TypeInfo::Array(_) => ui.label(format!("{name}: Array")),
                TypeInfo::Map(_) => ui.label(format!("{name}: Map")),
                TypeInfo::Enum(_) => ui.label(format!("{name}: Enum")),
                TypeInfo::Value(_) => ui.label(format!("{name}: Value")),
            };
        }
    } else {
        ui.label("Select an entity");
    }
}

fn get_reflected_component_data<'a>(
    entity: Entity,
    type_registry: &'a TypeRegistry,
    entities: &Entities,
    archetypes: &'a Archetypes,
    components: &'a Components,
) -> impl Iterator<Item = (&'a TypeInfo, &'a str)> + 'a {
    let entity_location = entities
        .get(entity)
        .unwrap_or_else(|| panic!("Entity {entity:?} does not exist"));

    let archetype = archetypes
        .get(entity_location.archetype_id)
        .unwrap_or_else(|| {
            panic!(
                "Archetype {:?} does not exist",
                entity_location.archetype_id
            )
        });

    archetype
        .components()
        .filter_map(|id| components.get_info(id))
        .filter_map(|component_info| {
            component_info
                .type_id()
                .map(|type_id| (type_id, component_info.name()))
        })
        .filter_map(|(type_id, name)| {
            type_registry
                .get_type_info(type_id)
                .map(|type_info| (type_info, name))
        })
}
