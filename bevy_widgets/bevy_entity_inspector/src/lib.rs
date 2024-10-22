//! This crate provides a entity inspector pane for Bevy Editor

use bevy::prelude::*;
use bevy_field_forms::FieldFormsPlugin;
use render::ChangeComponentField;
use render_impl::RenderStorage;

pub mod render;
pub mod render_impl;

pub struct EntityInspectorPlugin;

impl Plugin for EntityInspectorPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<FieldFormsPlugin>() {
            app.add_plugins(FieldFormsPlugin);
        }

        app.add_event::<ChangeComponentField>();

        app.init_resource::<RenderStorage>();
        app.add_plugins(render_impl::RenderImplPlugin);

        app.add_systems(PreUpdate, render::render_entity_inspector);
        app.add_systems(PreUpdate, render::render_component_inspector);

        app.add_observer(render::on_change_component_field);
    }
}

/// A marker for node in whicj entity inspector will render sub-tree

#[derive(Component)]
pub struct EntityInspector;

#[derive(Component)]
pub struct InspectedEntity;
