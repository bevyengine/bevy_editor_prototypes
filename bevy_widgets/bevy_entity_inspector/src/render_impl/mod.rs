//! This crate contains the implementation of some common components or types that are used in the entity inspector

pub mod float_impl;

use std::any::{Any, TypeId};

use bevy::{prelude::*, utils::HashMap};
use bevy_incomplete_bsn::entity_diff_tree::EntityDiffTree;

use crate::render::RenderContext;

pub struct RenderImplPlugin;

impl Plugin for RenderImplPlugin {
    fn build(&self, app: &mut App) {
        app.add_render_impl::<f32>(float_impl::float_render_impl);
    }
}

#[derive(Resource, Default)]
pub struct RenderStorage {
    pub renders: HashMap<TypeId, Box<dyn Fn(&dyn PartialReflect, &RenderContext) -> EntityDiffTree + Send + Sync + 'static>>,
}

pub trait RenderStorageApp {
    fn add_render_impl<T : Any + 'static>(&mut self, render_fn: impl Fn(&T, &RenderContext) -> EntityDiffTree + Send + Sync + 'static) -> &mut Self;
}

impl RenderStorageApp for App {
    fn add_render_impl<T : Any + 'static>(&mut self, render_fn: impl Fn(&T, &RenderContext) -> EntityDiffTree + Send + Sync + 'static) -> &mut Self {
        self.world_mut().resource_mut::<RenderStorage>().renders.insert(TypeId::of::<T>(), Box::new( move
            |untyped, context| render_fn(untyped.try_downcast_ref::<T>().unwrap(), context)
        ));
        self
    }
}
