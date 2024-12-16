//! This crate contains the implementation of some common components or types that are used in the entity inspector

pub mod float_impl;

use std::any::{Any, TypeId};

use bevy::{prelude::*, utils::HashMap};
use bevy_incomplete_bsn::entity_diff_tree::DiffTree;

use crate::render::RenderContext;

/// Plugin for store implementation for type to be rendered in entity inspector
pub struct RenderImplPlugin;

impl Plugin for RenderImplPlugin {
    fn build(&self, app: &mut App) {
        app.add_render_impl::<f32>(float_impl::float_render_impl);
    }
}

/// Storage for render-related data.
#[derive(Resource, Default)]
pub struct RenderStorage {
    /// Map of type IDs to render functions.
    pub renders: HashMap<
        TypeId,
        Box<
            dyn Fn(&dyn PartialReflect, String, &RenderContext) -> DiffTree + Send + Sync + 'static,
        >,
    >,
}

/// Trait for adding render implementations to an application.
pub trait RenderStorageApp {
    /// Adds a render implementation for a type to the render storage.
    fn add_render_impl<T: Any + 'static>(
        &mut self,
        render_fn: impl Fn(&T, String, &RenderContext) -> DiffTree + Send + Sync + 'static,
    ) -> &mut Self;
}

impl RenderStorageApp for App {
    fn add_render_impl<T: Any + 'static>(
        &mut self,
        render_fn: impl Fn(&T, String, &RenderContext) -> DiffTree + Send + Sync + 'static,
    ) -> &mut Self {
        self.world_mut()
            .resource_mut::<RenderStorage>()
            .renders
            .insert(
                TypeId::of::<T>(),
                Box::new(move |untyped, path, context| {
                    render_fn(untyped.try_downcast_ref::<T>().unwrap(), path, context)
                }),
            );
        self
    }
}
