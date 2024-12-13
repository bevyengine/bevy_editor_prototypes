//! Contains the system to render sub-tree for an entity inspector

#![allow(unsafe_code)]

use std::{any::TypeId, sync::Arc};

use bevy::{
    ecs::{
        component::{ComponentId, Components},
        system::SystemChangeTick,
    },
    prelude::*,
    reflect::ReflectFromPtr,
};
use bevy_collapsing_header::CollapsingHeader;
use bevy_editor_styles::Theme;
use bevy_incomplete_bsn::entity_diff_tree::{DiffTree, DiffTreeCommands};

use crate::{render_impl::RenderStorage, EntityInspector, InspectedEntity};

#[allow(dead_code)]
/// Context for rendering a component in the entity inspector.
pub struct RenderContext<'w, 's> {
    /// Storage for render-related data.
    render_storage: &'w RenderStorage,
    /// The entity being inspected.
    entity: Entity,
    /// The ID of the component being rendered.
    component_id: ComponentId,
    /// Theme to unify style
    theme: &'s Theme,
}

/// Component for managing the inspection of a specific component.
#[derive(Component)]
pub struct ComponentInspector {
    /// The ID of the component being inspected.
    pub component_id: ComponentId,
    /// The type ID of the component.
    pub type_id: TypeId,
    /// Flag indicating whether the component has been rendered.
    pub rendered: bool,
}

/// Component representing a change to a field in a component.
#[derive(Component, Clone)]
pub struct ChangeComponentField {
    /// The path to the field being changed.
    pub path: String,
    /// The new value for the field.
    pub value: Arc<dyn PartialReflect + Send + Sync + 'static>,
    /// Optional function for directly applying the change to the component.
    pub direct_cange:
        Option<Arc<dyn Fn(&mut dyn PartialReflect, &dyn PartialReflect) + Send + Sync + 'static>>,
}

impl Event for ChangeComponentField {
    type Traversal = &'static Parent;
    const AUTO_PROPAGATE: bool = true;
}

/// Renders the inspector for a specific component of an entity.
///
/// This system is responsible for updating the visual representation of a component
/// in the entity inspector UI when the component's data changes.
///
/// # Behavior
///
/// 1. Retrieves the currently inspected entity.
/// 2. Iterates through all ComponentInspector entities.
/// 3. For each ComponentInspector:
///    - Checks if the corresponding component on the inspected entity has changed.
///    - If changed, retrieves the component data and type information.
///    - Creates an EntityDiffTree to represent the updated UI for the component.
///    - Applies the updated UI to the inspector entity.
///
/// This system ensures that the entity inspector UI stays up-to-date with any changes
/// to the inspected entity's components, providing real-time feedback in the editor.

pub fn render_component_inspector(
    mut commands: Commands,
    mut q_inspector: Query<(Entity, &mut ComponentInspector), Without<InspectedEntity>>,
    q_inspected: Query<EntityRef, With<InspectedEntity>>,
    app_registry: Res<AppTypeRegistry>,
    system_change_ticks: SystemChangeTick,
    render_storage: Res<RenderStorage>,
    theme: Res<Theme>,
) {
    let Ok(inspected_entity) = q_inspected.get_single() else {
        warn!("No inspected entity or many found");
        return;
    };

    for (inspector_entity, mut inspector) in q_inspector.iter_mut() {
        let Some(change_ticks) = inspected_entity.get_change_ticks_by_id(inspector.component_id)
        else {
            warn!("No change ticks found for component: {:?}", inspector.component_id);
            continue;
        };

        if !change_ticks.is_changed(
            system_change_ticks.last_run(),
            system_change_ticks.this_run(),
        ) && inspector.rendered
        {
            info!("Component not changed for component: {:?}, skipping render", inspector.component_id);
            continue;
        }

        // Component was changed, render it

        let type_registry = app_registry.read();

        let Some(reg) = type_registry.get(inspector.type_id) else {
            warn!("No type registry found for type: {:?}", inspector.type_id);
            continue;
        };

        let Some(reflect_from_ptr) = reg.data::<ReflectFromPtr>() else {
            warn!("No ReflectFromPtr found for type: {:?}", inspector.type_id);
            continue;
        };

        let Ok(component_data) = inspected_entity.get_by_id(inspector.component_id) else {
            warn!("No component data found for component: {:?}", inspector.component_id);
            continue;
        };

        let reflected_data = unsafe { reflect_from_ptr.from_ptr()(component_data) };

        let mut tree = DiffTree::new().with_patch_fn(|node: &mut Node| {
            node.flex_direction = FlexDirection::Column;
            node.max_width = Val::Px(300.0);
        });

        let name = reg
            .type_info()
            .type_path()
            .split("::")
            .last()
            .unwrap_or_default();

        let render_context = RenderContext {
            render_storage: &render_storage,
            entity: inspected_entity.id(),
            component_id: inspector.component_id,
            theme: &theme,
        };

        let reflect_content = recursive_reflect_render(
            reflected_data.as_partial_reflect(),
            format!(""), // The string reflect path starts with a dot
            &render_context,
        );

        tree.add_child(
            DiffTree::new()
                .with_patch_fn(move |collapsing_header: &mut CollapsingHeader| {
                    collapsing_header.text = name.to_string();
                })
                .with_patch_fn(|text_layout: &mut TextLayout| {
                    text_layout.linebreak = LineBreak::AnyCharacter;
                })
                .with_patch_fn(|node: &mut Node| {
                    node.max_width = Val::Px(300.0);
                })
                .with_child(reflect_content),
        );

        let id = inspector.component_id;

        // tree.add_child(
        //     DiffTree::new()
        //         .with_patch_fn(move |text: &mut Text| {
        //             text.0 = format!("Component: {:?}", id);
        //         })
        // );

        let font_cloned = theme.text.font.clone();
        let color_cloned = theme.text.text_color;
        tree.add_cascade_patch_fn::<TextFont, Text>(move |font: &mut TextFont| {
            font.font = font_cloned.clone();
            font.font_size = 14.0;
        });
        tree.add_cascade_patch_fn::<TextColor, Text>(move |color: &mut TextColor| {
            color.0 = color_cloned.clone();
        });

        commands.entity(inspector_entity).diff_tree(tree);

        inspector.rendered = true;
    }
}

/// Observer for change component field event
pub fn on_change_component_field(
    trigger: Trigger<ChangeComponentField>,
    q_component_inspectors: Query<&ComponentInspector, Without<InspectedEntity>>,
    mut q_inspected: Query<EntityMut, With<InspectedEntity>>,
    app_registry: Res<AppTypeRegistry>,
) {
    let entity = trigger.entity();
    let Ok(inspector) = q_component_inspectors.get(entity) else {
        return;
    };

    let Ok(mut inspected_entity) = q_inspected.get_single_mut() else {
        error!("No inspected entity found");
        return;
    };

    let type_registry = app_registry.read();

    let Some(reg) = type_registry.get(inspector.type_id) else {
        error!("No type registry found for type: {:?}", inspector.type_id);
        return;
    };

    let Some(reflect_from_ptr) = reg.data::<ReflectFromPtr>() else {
        error!("No ReflectFromPtr found for type: {:?}", inspector.type_id);
        return;
    };

    let Ok(mut component_data) = inspected_entity.get_mut_by_id(inspector.component_id) else {
        error!("Failed to get component data");
        return;
    };

    {
        let reflected_data = unsafe { reflect_from_ptr.from_ptr_mut()(component_data.as_mut()) };

        let Ok(field) = reflected_data.reflect_path_mut(trigger.path.as_str()) else {
            error!("Failed to reflect path: {:?}", trigger.path);
            return;
        };

        if let Some(direct_change) = trigger.direct_cange.as_ref() {
            info!("Apply direct change to field: {:?}", trigger.path);
            direct_change(field, trigger.value.as_ref());
        } else {
            info!("Apply value to field: {:?}", trigger.path);
            field.apply(trigger.value.as_ref());
        }
    }

    component_data.set_changed();
}

/// Render the entity inspector
pub fn render_entity_inspector(
    mut commands: Commands,
    q_inspected: Query<EntityRef, With<InspectedEntity>>,
    mut q_inspector: Query<(Entity, Option<&Children>, &mut Node), (Without<InspectedEntity>, With<EntityInspector>)>,
    q_component_inspectors: Query<&ComponentInspector>,
    components: &Components,
) {
    let Ok(inspected_entity) = q_inspected.get_single() else {
        return;
    };

    for (inspector, children, mut node) in q_inspector.iter_mut() {
        let entity = inspected_entity.id();

        node.display = Display::Flex;
        node.flex_direction = FlexDirection::Column;
        node.overflow = Overflow::scroll();
        node.height = Val::Percent(100.0);

        // let mut tree = DiffTree::new();

        // tree.add_patch_fn(|node: &mut Node| {
        //     node.display = Display::Flex;
        //     node.flex_direction = FlexDirection::Column;
        //     node.overflow = Overflow::scroll();
        //     node.height = Val::Percent(100.0);
        // });

        // tree.add_patch_fn(|_: &mut Interaction| {});

        // tree.add_child(DiffTree::new().with_patch_fn(move |text: &mut Text| {
        //     text.0 = format!("Entity: {}", entity);
        // }));
        // commands.entity(inspector).diff_tree(tree);

        let mut compenent_id_set = inspected_entity
            .archetype()
            .components()
            .collect::<Vec<_>>();

        let mut found_component_ids = Vec::new();

        if let Some(children) = children {
            for child in children.iter() {
                let Ok(component_inspector) = q_component_inspectors.get(*child) else {
                    continue;
                };

                if compenent_id_set.contains(&component_inspector.component_id) {
                    found_component_ids.push(component_inspector.component_id);
                } else {
                    // Component is not attached to the entity anymore, remove it
                    info!(
                        "Component is not attached to the entity anymore, removing it: {:?}",
                        component_inspector.component_id
                    );
                    commands.entity(*child).despawn_recursive();
                }
            }
        }

        // Find the components that are not represented in the inspector
        compenent_id_set.retain(|id| !found_component_ids.contains(id));
        // Add new inspectors for the remaining components
        for component_id in compenent_id_set.iter() {
            let Some(info) = components.get_info(*component_id) else {
                continue;
            };

            let Some(type_id) = info.type_id() else {
                continue;
            };

            let component_inspector_entity = commands
                .spawn(ComponentInspector {
                    component_id: *component_id,
                    type_id,
                    rendered: false,
                })
                .id();

            commands
                .entity(inspector)
                .add_child(component_inspector_entity);
        }

    }
}

fn recursive_reflect_render(
    data: &dyn PartialReflect,
    path: String,
    render_context: &RenderContext,
) -> DiffTree {
    if let Some(render_fn) = render_context
        .render_storage
        .renders
        .get(&data.get_represented_type_info().unwrap().type_id())
    {
        return render_fn(data, path, render_context);
    } else {
        let mut tree = DiffTree::new();
        tree.add_patch_fn(|node: &mut Node| {
            node.display = Display::Flex;
            node.flex_direction = FlexDirection::Column;
        });
        match data.reflect_ref() {
            bevy::reflect::ReflectRef::Struct(v) => {
                for field_idx in 0..v.field_len() {
                    let field = v.field_at(field_idx).unwrap();
                    let name = v.name_at(field_idx).unwrap_or_default().to_string();
                    if field.reflect_ref().as_opaque().is_ok() {
                        // Opaque fields are rendered as a row
                        let mut row = DiffTree::new().with_patch_fn(|node: &mut Node| {
                            node.flex_direction = FlexDirection::Row;
                        });
                        let moving_name = name.clone();
                        row.add_child(
                            DiffTree::new()
                                .with_patch_fn(move |text: &mut Text| {
                                    text.0 = format!("{}", moving_name);
                                })
                                .with_patch_fn(|node: &mut Node| {
                                    node.padding = UiRect::all(Val::Px(2.0));
                                }),
                        );
                        row.add_child(recursive_reflect_render(
                            field,
                            format!("{}.{}", path, name),
                            render_context,
                        ));
                        tree.add_child(row);
                    } else {
                        // Other fields are rendered as a column with a shift
                        let moving_name = name.clone();
                        tree.add_child(
                            DiffTree::new()
                                .with_patch_fn(move |text: &mut Text| {
                                    text.0 = format!("{}", moving_name);
                                })
                                .with_patch_fn(|node: &mut Node| {
                                    node.margin = UiRect::all(Val::Px(5.0));
                                }),
                        );

                        let mut row = DiffTree::new().with_patch_fn(|node: &mut Node| {
                            node.flex_direction = FlexDirection::Row;
                        });

                        // Add tab
                        row.add_child(DiffTree::new().with_patch_fn(|node: &mut Node| {
                            node.width = Val::Px(20.0);
                        }));

                        row.add_child(recursive_reflect_render(
                            field,
                            format!("{}.{}", path, name),
                            render_context,
                        ));

                        tree.add_child(row);
                    }
                }
            }
            bevy::reflect::ReflectRef::TupleStruct(v) => {
                for (idx, field) in v.iter_fields().enumerate() {
                    tree.add_child(recursive_reflect_render(
                        field,
                        format!("{}[{}]", path, idx),
                        render_context,
                    ));
                }
            }
            bevy::reflect::ReflectRef::Tuple(v) => {
                for (idx, field) in v.iter_fields().enumerate() {
                    tree.add_child(recursive_reflect_render(
                        field,
                        format!("{}[{}]", path, idx),
                        render_context,
                    ));
                }
            }
            bevy::reflect::ReflectRef::List(v) => {
                for (idx, field) in v.iter().enumerate() {
                    tree.add_child(recursive_reflect_render(
                        field,
                        format!("{}[{}]", path, idx),
                        render_context,
                    ));
                }
            }
            bevy::reflect::ReflectRef::Array(v) => {
                for (idx, field) in v.iter().enumerate() {
                    tree.add_child(recursive_reflect_render(
                        field,
                        format!("{}[{}]", path, idx),
                        render_context,
                    ));
                }
            }
            bevy::reflect::ReflectRef::Map(v) => {
                for (_, field) in v.iter() {
                    tree.add_child(recursive_reflect_render(
                        field,
                        format!("{}", path),
                        render_context,
                    ));
                }
            }
            bevy::reflect::ReflectRef::Set(v) => {
                for field in v.iter() {
                    tree.add_child(recursive_reflect_render(
                        field,
                        format!("{}", path),
                        render_context,
                    ));
                }
            }
            bevy::reflect::ReflectRef::Enum(v) => {
                for field in v.iter_fields() {
                    tree.add_child(recursive_reflect_render(
                        field.value(),
                        format!("{}", path),
                        render_context,
                    ));
                }
            }
            bevy::reflect::ReflectRef::Opaque(v) => {
                let v = v.clone_value();
                let font_cloned = render_context.theme.text.font.clone();
                let color_cloned = render_context.theme.text.text_color;
                tree.add_child(
                    DiffTree::new()
                        .with_patch_fn(move |text: &mut Text| {
                            text.0 = format!("{:?}", v);
                        })
                        .with_patch_fn(move |font: &mut TextFont| {
                            font.font = font_cloned.clone();
                        })
                        .with_patch_fn(move |color: &mut TextColor| {
                            color.0 = color_cloned.clone();
                        })
                        .with_patch_fn(|node: &mut Node| {
                            node.padding = UiRect::all(Val::Px(2.0));
                        }),
                );
            }
        }
        tree
    }
}
