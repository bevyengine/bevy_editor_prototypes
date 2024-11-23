use std::collections::VecDeque;

use bevy::{
    app::{App, First, Main, MainSchedulePlugin, PluginsState, Update},
    asset::{AssetId, AssetPlugin, AssetServer, Assets, Handle},
    core::{FrameCountPlugin, TaskPoolPlugin, TypeRegistrationPlugin},
    core_pipeline::CorePipelinePlugin,
    diagnostic::LogDiagnosticsPlugin,
    ecs::{
        entity::EntityHashMap,
        event::{event_update_condition, event_update_system, EventUpdates},
        query::QuerySingleError,
        schedule::ScheduleLabel,
        world,
    },
    gltf::GltfAssetLabel,
    log::{error, info, LogPlugin},
    math::{UVec2, Vec3},
    pbr::{DirectionalLight, MeshMaterial3d, PbrPlugin, StandardMaterial},
    prelude::{
        AppTypeRegistry, Camera, Camera3d, Commands, Component, Deref, DerefMut,
        DespawnRecursiveExt, Entity, Event, EventReader, EventWriter, FromWorld, Image,
        ImagePlugin, IntoSystemConfigs, Mesh, Mesh3d, NonSendMut, PluginGroup, Query, Res, ResMut,
        Resource, Transform, With, World,
    },
    render::{
        camera::RenderTarget,
        pipelined_rendering::PipelinedRenderingPlugin,
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
        renderer::RenderDevice,
        view::RenderLayers,
        Extract, ExtractSchedule, RenderApp, RenderPlugin,
    },
    scene::{InstanceId, Scene, SceneInstance, SceneRoot, SceneSpawner},
    time::TimePlugin,
    ui::{IsDefaultUiCamera, TargetCamera},
    utils::{Entry, HashMap, HashSet},
    window::{WindowClosing, WindowCreated, WindowPlugin, WindowResized},
    winit::WinitPlugin,
    DefaultPlugins, MinimalPlugins,
};

use crate::PreviewAsset;

pub const BASE_PREVIEW_LAYER: usize = 128;
pub const PREVIEW_LAYERS_COUNT: usize = 8;
pub const PREVIEW_RENDER_FRAMES: u32 = 32;

#[derive(Resource)]
pub struct PreviewRenderLayers {
    available: u64,
}

impl Default for PreviewRenderLayers {
    fn default() -> Self {
        Self { available: !0 }
    }
}

impl PreviewRenderLayers {
    pub fn occupy(&mut self) -> Option<usize> {
        if self.is_full() {
            None
        } else {
            let n = self.available.trailing_zeros() as usize;
            self.available &= !(1 << n);
            Some(n)
        }
    }

    pub fn free(&mut self, layer: RenderLayers) {
        for b in layer.iter() {
            self.available &= !(1 << b);
        }
    }

    pub fn is_full(&self) -> bool {
        self.available.trailing_zeros() == PREVIEW_LAYERS_COUNT as u32
    }
}

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
        &[255, 0, 0, 255],
        TextureFormat::Bgra8UnormSrgb,
        Default::default(),
    );

    image.texture_descriptor.usage |=
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

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
    cameras: [Entity; PREVIEW_LAYERS_COUNT],
    handles: [Handle<Scene>; PREVIEW_LAYERS_COUNT],
    instances: [Option<InstanceId>; PREVIEW_LAYERS_COUNT],
    applied_layer: [bool; PREVIEW_LAYERS_COUNT],
}

impl FromWorld for PreviewSceneState {
    fn from_world(world: &mut World) -> Self {
        let mut cameras = [Entity::PLACEHOLDER; PREVIEW_LAYERS_COUNT];

        for i in 0..PREVIEW_LAYERS_COUNT {
            world.spawn((
                DirectionalLight::default(),
                Transform::IDENTITY.looking_to(Vec3::new(1.0, -1.0, 1.0), Vec3::Y),
                RenderLayers::from_layers(&[i + BASE_PREVIEW_LAYER]),
            ));

            cameras[i] = world
                .spawn((
                    Camera3d::default(),
                    Camera {
                        target: RenderTarget::Image(Handle::default()),
                        is_active: false,
                        ..Default::default()
                    },
                    Transform::from_translation(Vec3::new(-5.0, 2.0, -5.0))
                        .looking_at(Vec3::ZERO, Vec3::Y),
                    PreviewRenderView { layer: i },
                    RenderLayers::from_layers(&[i + BASE_PREVIEW_LAYER]),
                ))
                .id();
        }

        Self {
            cameras,
            handles: std::array::from_fn(|_| Handle::default()),
            instances: [None; PREVIEW_LAYERS_COUNT],
            applied_layer: [false; PREVIEW_LAYERS_COUNT],
        }
    }
}

/// Scenes that are rendered for preview purpose. This should be inserted into
/// main world.
#[derive(Resource, Default)]
pub struct PrerenderedScenes {
    rendered: HashMap<AssetId<Scene>, Handle<Image>>,
    rendering: HashSet<AssetId<Scene>>,
    queue: HashSet<Handle<Scene>>,
}

impl PrerenderedScenes {
    pub fn get_or_schedule(&mut self, handle: Handle<Scene>) -> Option<Handle<Image>> {
        let id = handle.id();
        match self.rendered.entry(id) {
            Entry::Occupied(e) => Some(e.get().clone()),
            Entry::Vacant(_) => {
                if !self.rendering.contains(&id) {
                    self.queue.insert(handle);
                    self.rendering.insert(id);
                }
                None
            }
        }
    }
}

pub(crate) fn update_queue(
    mut commands: Commands,
    mut prerendered: ResMut<PrerenderedScenes>,
    mut render_layers: ResMut<PreviewRenderLayers>,
    mut scene_spawner: ResMut<SceneSpawner>,
    mut scene_state: ResMut<PreviewSceneState>,
    mut camera_query: Query<&mut Camera, With<PreviewRenderView>>,
    settings: Res<PreviewSettings>,
    mut images: ResMut<Assets<Image>>,
    mut preview_rendered: EventReader<PreviewRendered>,
) {
    while !render_layers.is_full() {
        let Some(handle) = prerendered.queue.iter().nth(0).take().cloned() else {
            dbg!();
            break;
        };
        dbg!(&handle);

        let instance = scene_spawner.spawn(handle.clone());
        let layer = render_layers.occupy().unwrap();
        scene_state.handles[layer] = handle;
        scene_state.instances[layer] = Some(instance);
        scene_state.applied_layer[layer] = false;

        let camera_entity = scene_state.cameras[layer];
        let mut camera = camera_query.get_mut(camera_entity).unwrap();
        camera.is_active = true;
        camera.target = RenderTarget::Image(images.add(create_prerender_target(&settings)));
        commands
            .entity(camera_entity)
            .insert(PreviewRenderedFrames::default());
    }

    for finished in preview_rendered.read() {
        let mut camera = camera_query
            .get_mut(scene_state.cameras[finished.layer])
            .unwrap();
        camera.is_active = false;
        let RenderTarget::Image(target) = &camera.target else {
            unreachable!()
        };

        render_layers.free(RenderLayers::from_layers(&[finished.layer]));
        let handle = scene_state.handles[finished.layer].clone();
        dbg!(&handle);
        prerendered.rendering.remove(&handle.id());
        prerendered.rendered.insert(handle.id(), target.clone());
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
        if scene_state.instances[view.layer]
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
        if let Some(instance) = scene_state.instances[layer] {
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
