use bevy::{
    asset::Handle,
    prelude::{Deref, DerefMut, Image, Mesh, Resource},
    utils::HashMap,
};

/// Meshes that are rendered for preview purpose. This should be inserted into
/// main world.
#[derive(Resource, Default, Deref, DerefMut)]
pub struct PrerenderedMesh(HashMap<Handle<Mesh>, Handle<Image>>);
