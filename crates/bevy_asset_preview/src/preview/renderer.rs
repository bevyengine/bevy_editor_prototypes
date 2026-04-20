use bevy::{
    camera::{
        Camera3d, RenderTarget,
        primitives::{Aabb, Sphere as CameraSphere},
        visibility::RenderLayers,
    },
    prelude::*,
    render::render_resource::TextureFormat,
};

use crate::preview::PreviewConfig;

/// Creates a render target texture for preview rendering.
pub fn setup_render_target(images: &mut Assets<Image>, resolution: u32) -> Handle<Image> {
    let image = Image::new_target_texture(resolution, resolution, TextureFormat::bevy_default());
    images.add(image)
}

/// Creates preview camera that renders to the specified target.
pub fn create_preview_camera(
    commands: &mut Commands,
    render_target: Handle<Image>,
    render_layer: usize,
) -> Entity {
    let render_layers = RenderLayers::layer(render_layer);

    commands
        .spawn((
            Camera3d::default(),
            Camera {
                target: RenderTarget::Image(render_target.into()),
                order: -1, // Render before main camera
                ..default()
            },
            render_layers,
        ))
        .id()
}

/// Creates preview lights (ambient + directional).
pub fn create_preview_lights(commands: &mut Commands, render_layer: usize) {
    let render_layers = RenderLayers::layer(render_layer);

    // Ambient light
    commands.spawn((
        AmbientLight {
            brightness: 0.3,
            ..default()
        },
        render_layers.clone(),
    ));

    // Directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
        render_layers,
    ));
}

/// Calculates the bounding box of entities in a query.
pub fn calculate_bounds(
    query: &Query<(&GlobalTransform, Option<&Aabb>), (With<Mesh3d>, Without<Camera>)>,
) -> Option<Aabb> {
    if query.iter().any(|(_, maybe_aabb)| maybe_aabb.is_none()) {
        return None;
    }

    let mut min = Vec3A::splat(f32::MAX);
    let mut max = Vec3A::splat(f32::MIN);
    let mut has_any = false;

    for (transform, maybe_aabb) in query.iter() {
        let Some(aabb) = maybe_aabb else {
            continue;
        };

        has_any = true;

        let sphere = CameraSphere {
            center: Vec3A::from(transform.transform_point(Vec3::from(aabb.center))),
            radius: transform.radius_vec3a(aabb.half_extents),
        };
        let transformed_aabb = Aabb::from(sphere);
        min = min.min(transformed_aabb.min());
        max = max.max(transformed_aabb.max());
    }

    if has_any {
        Some(Aabb::from_min_max(Vec3::from(min), Vec3::from(max)))
    } else {
        None
    }
}

/// Positions camera to view the entire bounding box.
pub fn position_camera_for_bounds(camera_transform: &mut Transform, bounds: &Aabb) {
    let center = Vec3::from(bounds.center);
    let size = (Vec3::from(bounds.max()) - Vec3::from(bounds.min())).length();

    camera_transform.translation = center + size * Vec3::new(0.5, 0.25, 0.5);
    camera_transform.look_at(center, Vec3::Y);
}

/// Spawns a model scene preview entity (for GLTF, OBJ, FBX, etc.).
pub fn spawn_model_scene_preview(
    commands: &mut Commands,
    scene_handle: Handle<Scene>,
    render_layer: usize,
) -> Entity {
    let render_layers = RenderLayers::layer(render_layer);

    commands
        .spawn((SceneRoot(scene_handle), render_layers))
        .id()
}

/// Spawns a material preview entity (sphere with material).
pub fn spawn_material_preview(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material_handle: Handle<StandardMaterial>,
    render_layer: usize,
) -> Entity {
    let render_layers = RenderLayers::layer(render_layer);

    // Create a sphere mesh for material preview
    let sphere_mesh = meshes.add(Sphere::default().mesh().uv(32, 18));

    commands
        .spawn((
            Mesh3d(sphere_mesh),
            MeshMaterial3d(material_handle),
            Transform::default(),
            render_layers,
        ))
        .id()
}

/// Spawns a mesh preview entity (mesh with default material).
pub fn spawn_mesh_preview(
    commands: &mut Commands,
    materials: &mut Assets<StandardMaterial>,
    mesh_handle: Handle<Mesh>,
    render_layer: usize,
) -> Entity {
    let render_layers = RenderLayers::layer(render_layer);

    // Create default material
    let default_material = materials.add(StandardMaterial::default());

    commands
        .spawn((
            Mesh3d(mesh_handle),
            MeshMaterial3d(default_material),
            Transform::default(),
            render_layers,
        ))
        .id()
}

/// Creates a complete preview scene with camera, lights, and asset.
pub fn create_preview_scene(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    config: &PreviewConfig,
    resolution: u32,
) -> (Entity, Handle<Image>, Entity) {
    // Create render target
    let render_target = setup_render_target(images, resolution);

    // Create camera
    let camera_entity = create_preview_camera(commands, render_target.clone(), config.render_layer);

    // Create lights
    create_preview_lights(commands, config.render_layer);

    (camera_entity, render_target, camera_entity)
}
