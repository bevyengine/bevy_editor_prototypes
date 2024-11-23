use bevy::{
    asset::{AssetId, Assets, Handle},
    math::{UVec2, Vec3},
    pbr::DirectionalLight,
    prelude::{
        Camera, Camera3d, Commands, Component, Entity, Event, EventReader, EventWriter, Image,
        Query, Res, ResMut, Resource, Transform,
    },
    render::{
        camera::RenderTarget,
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
        renderer::RenderDevice,
        texture::BevyDefault,
        view::RenderLayers,
    },
    scene::{InstanceId, Scene, SceneSpawner},
    utils::{Entry, HashMap, HashSet},
};

use crate::render::receive::{PreviewImageCopies, PreviewImageCopy};

pub const BASE_PREVIEW_LAYER: usize = 128;
pub const PREVIEW_LAYERS_COUNT: usize = 8;
pub const PREVIEW_RENDER_FRAMES: u32 = 32;

pub mod receive;

#[derive(Resource)]
pub struct PreviewSettings {
    pub resolution: UVec2,
}

impl Default for PreviewSettings {
    fn default() -> Self {
        Self {
            resolution: UVec2::splat(256),
        }
    }
}

fn create_prerender_target(settings: &PreviewSettings) -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width: settings.resolution.x,
            height: settings.resolution.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::bevy_default(),
        Default::default(),
    );

    image.texture_descriptor.usage |= TextureUsages::TEXTURE_BINDING
        | TextureUsages::COPY_DST
        | TextureUsages::COPY_SRC
        | TextureUsages::RENDER_ATTACHMENT;

    image
}

#[derive(Component)]
pub struct PreviewRenderView {
    pub layer: usize,
}

#[derive(Component, Default)]
pub struct PreviewRenderedFrames {
    pub cur_frame: u32,
}

#[derive(Event)]
pub struct PreviewRendered {
    pub layer: usize,
}

#[derive(Resource)]
pub struct PreviewSceneState {
    available_layers: u8,
    cameras: [Entity; PREVIEW_LAYERS_COUNT],
    lights: [Entity; PREVIEW_LAYERS_COUNT],
    scene_handles: [Handle<Scene>; PREVIEW_LAYERS_COUNT],
    scene_instances: [Option<InstanceId>; PREVIEW_LAYERS_COUNT],
    applied_layer: [bool; PREVIEW_LAYERS_COUNT],
    render_targets: [Handle<Image>; PREVIEW_LAYERS_COUNT],
}

impl Default for PreviewSceneState {
    fn default() -> Self {
        Self {
            available_layers: !0,
            cameras: [Entity::PLACEHOLDER; PREVIEW_LAYERS_COUNT],
            lights: [Entity::PLACEHOLDER; PREVIEW_LAYERS_COUNT],
            scene_handles: Default::default(),
            scene_instances: Default::default(),
            applied_layer: Default::default(),
            render_targets: Default::default(),
        }
    }
}

impl PreviewSceneState {
    pub fn occupy(
        &mut self,
        handle: Handle<Scene>,
        instance: InstanceId,
        render_target: Handle<Image>,
        commands: &mut Commands,
    ) {
        if self.is_full() {
            return;
        }

        let layer = self.available_layers.trailing_zeros() as usize;
        self.available_layers &= !(1 << layer);

        self.lights[layer] = commands
            .spawn((
                DirectionalLight::default(),
                Transform::IDENTITY.looking_to(Vec3::new(1.0, -1.0, 1.0), Vec3::Y),
                RenderLayers::from_layers(&[layer + BASE_PREVIEW_LAYER]),
            ))
            .id();
        self.cameras[layer] = commands
            .spawn((
                Camera3d::default(),
                Camera {
                    target: RenderTarget::Image(render_target.clone()),
                    ..Default::default()
                },
                Transform::from_translation(Vec3::new(-5.0, 2.0, -5.0))
                    .looking_at(Vec3::ZERO, Vec3::Y),
                RenderLayers::from_layers(&[layer + BASE_PREVIEW_LAYER]),
                PreviewRenderView { layer },
                PreviewRenderedFrames::default(),
            ))
            .id();
        self.render_targets[layer] = render_target;
        self.scene_handles[layer] = handle;
        self.scene_instances[layer] = Some(instance);
    }

    pub fn free(&mut self, layer: usize, commands: &mut Commands) {
        self.available_layers |= 1 << layer;
        commands.entity(self.lights[layer]).despawn();
        commands.entity(self.cameras[layer]).despawn();
        self.applied_layer[layer] = false;
        self.scene_instances[layer] = None;
    }

    pub fn is_full(&self) -> bool {
        self.available_layers.trailing_zeros() == PREVIEW_LAYERS_COUNT as u32
    }
}

/// Scenes that are rendered for preview purpose. This should be inserted into
/// main world.
#[derive(Resource, Default)]
pub struct RenderedScenePreviews {
    pub(crate) changed: HashMap<AssetId<Image>, AssetId<Scene>>,
    pub(crate) available: HashMap<AssetId<Scene>, Handle<Image>>,
    pub(crate) rendering: HashSet<AssetId<Scene>>,
    pub(crate) queue: HashSet<Handle<Scene>>,
}

impl RenderedScenePreviews {
    pub fn get_or_schedule(&mut self, handle: Handle<Scene>) -> Option<Handle<Image>> {
        let id = handle.id();
        match self.available.entry(id) {
            Entry::Occupied(e) => Some(e.get().clone()),
            Entry::Vacant(_) => {
                if !self.rendering.contains(&id) {
                    self.queue.insert(handle);
                    self.rendering.insert(id);
                } else {
                }
                None
            }
        }
    }
}

pub(crate) fn update_queue(
    mut commands: Commands,
    mut previews: ResMut<RenderedScenePreviews>,
    mut scene_spawner: ResMut<SceneSpawner>,
    mut scene_state: ResMut<PreviewSceneState>,
    settings: Res<PreviewSettings>,
    mut images: ResMut<Assets<Image>>,
    mut preview_rendered: EventReader<PreviewRendered>,
    mut image_copies: ResMut<PreviewImageCopies>,
    render_device: Res<RenderDevice>,
) {
    while !scene_state.is_full() {
        let Some(handle) = previews.queue.iter().nth(0).cloned() else {
            break;
        };
        previews.queue.remove(&handle);

        let instance = scene_spawner.spawn(handle.clone());
        let render_target = images.add(create_prerender_target(&settings));
        scene_state.occupy(handle, instance, render_target, &mut commands);
    }

    for finished in preview_rendered.read() {
        let scene_handle = scene_state.scene_handles[finished.layer].clone();
        previews.rendering.remove(&scene_handle.id());
        let render_target = scene_state.render_targets[finished.layer].clone();
        image_copies.insert(
            render_target.id(),
            PreviewImageCopy::new(settings.resolution.x, settings.resolution.y, &render_device),
        );
        previews
            .changed
            .insert(render_target.id(), scene_handle.id());
        previews.available.insert(scene_handle.id(), render_target);

        let instance = scene_state.scene_instances[finished.layer].unwrap();
        scene_spawner.despawn_instance(instance);
        scene_state.free(finished.layer, &mut commands);
    }
}

pub(crate) fn update_preview_frames_counter(
    mut commands: Commands,
    mut counters_query: Query<(Entity, &mut PreviewRenderedFrames, &PreviewRenderView)>,
    mut preview_rendered: EventWriter<PreviewRendered>,
    scene_state: Res<PreviewSceneState>,
    scene_spawner: Res<SceneSpawner>,
) {
    for (entity, mut cnt, view) in &mut counters_query {
        if scene_state.scene_instances[view.layer]
            .is_some_and(|inst| scene_spawner.instance_is_ready(inst))
        {
            cnt.cur_frame += 1;

            if cnt.cur_frame >= PREVIEW_RENDER_FRAMES {
                commands.entity(entity).remove::<PreviewRenderedFrames>();
                preview_rendered.send(PreviewRendered { layer: view.layer });
            }
        }
    }
}

pub(crate) fn change_render_layers(
    mut commands: Commands,
    mut scene_state: ResMut<PreviewSceneState>,
    scene_spawner: Res<SceneSpawner>,
) {
    for layer in 0..PREVIEW_LAYERS_COUNT {
        if let Some(instance) = scene_state.scene_instances[layer] {
            if !scene_state.applied_layer[layer] && scene_spawner.instance_is_ready(instance) {
                scene_state.applied_layer[layer] = true;

                commands.insert_batch(
                    scene_spawner
                        .iter_instance_entities(instance)
                        .map(|e| (e, RenderLayers::from_layers(&[layer + BASE_PREVIEW_LAYER])))
                        .collect::<Vec<_>>(),
                );
            }
        }
    }
}
