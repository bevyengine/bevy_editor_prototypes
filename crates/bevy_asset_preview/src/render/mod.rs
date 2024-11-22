use std::collections::VecDeque;

use bevy::{
    asset::{AssetId, Assets, Handle},
    core_pipeline::core_3d::MainOpaquePass3dNode,
    ecs::{query::QueryItem, system::lifetimeless::Read},
    pbr::{MeshMaterial3d, StandardMaterial},
    prelude::{
        Camera, Camera3d, Commands, Component, Deref, FromWorld, Image, Mesh, Mesh3d, Query,
        QueryState, Res, ResMut, Resource, World,
    },
    render::{
        camera::RenderTarget,
        extract_component::ExtractComponent,
        extract_resource::ExtractResource,
        render_graph::{Node, NodeRunError, RenderGraphContext, ViewNode},
        renderer::RenderContext,
        view::RenderLayers,
    },
    utils::{Entry, HashMap, HashSet},
};

pub const PRERENDER_LAYER: RenderLayers = RenderLayers::layer(63);

#[derive(Component, Clone)]
pub struct PrerenderMesh;

#[derive(Component)]
pub struct PrerendererView;

pub fn setup_prerender_scene(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            target: RenderTarget::Image(Handle::default()),
            ..Default::default()
        },
        PRERENDER_LAYER,
    ));
}

/// Meshes that are rendered for preview purpose. This should be inserted into
/// main world.
#[derive(Resource, Default)]
pub struct PrerenderedMeshes {
    rendered: HashMap<AssetId<Mesh>, Handle<Image>>,
    queue: VecDeque<Handle<Mesh>>,
}

impl PrerenderedMeshes {
    pub fn get_or_schedule(&mut self, handle: Handle<Mesh>) -> Option<Handle<Image>> {
        let id = handle.id();
        match self.rendered.entry(id) {
            Entry::Occupied(e) => Some(e.get().clone()),
            Entry::Vacant(_) => {
                self.queue.push_back(handle);
                None
            }
        }
    }
}

#[derive(Resource, Deref)]
pub struct PrerenderedMeshMaterial(MeshMaterial3d<StandardMaterial>);

impl FromWorld for PrerenderedMeshMaterial {
    fn from_world(world: &mut World) -> Self {
        // TODO materialize once user applied material or detected material in model
        Self(MeshMaterial3d(
            world
                .resource_mut::<Assets<StandardMaterial>>()
                .add(StandardMaterial::default()),
        ))
    }
}

pub(crate) fn flush_queue(
    mut commands: Commands,
    mut meshes: ResMut<PrerenderedMeshes>,
    default_material: Res<PrerenderedMeshMaterial>,
) {
    let Some(mesh) = meshes.queue.pop_front() else {
        return;
    };
}
