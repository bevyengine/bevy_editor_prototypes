use bevy::camera::primitives::Aabb;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_editor_core::selection::EditorSelection;

#[derive(SystemParam)]
pub struct SelectionBoxQueries<'w, 's> {
    pub mesh_query: Query<
        'w,
        's,
        (
            &'static GlobalTransform,
            &'static Mesh3d,
            Option<&'static Aabb>,
        ),
    >,
    pub sprite_query: Query<'w, 's, (&'static GlobalTransform, &'static Sprite), Without<Mesh3d>>,
    pub aabb_query: Query<
        'w,
        's,
        (&'static GlobalTransform, &'static Aabb),
        (Without<Mesh3d>, Without<Sprite>),
    >,
    pub children_query: Query<'w, 's, &'static Children>,
    pub transform_query:
        Query<'w, 's, &'static GlobalTransform, (Without<Mesh3d>, Without<Sprite>, Without<Aabb>)>,
}

pub struct SelectionBoxPlugin;
impl Plugin for SelectionBoxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShowSelectionBox>()
            .add_systems(Startup, spawn_selection_box_toggle_ui)
            .add_systems(Update, selection_box_system)
            .add_systems(Update, update_selection_box_toggle_text);
    }
}

#[derive(Resource, Default)]
pub struct ShowSelectionBox(pub bool);

// Marker for the toggle button text
#[derive(Component)]
struct SelectionBoxToggleText;

/// Draw an outline for a world-space AABB
fn draw_selection_box(gizmos: &mut Gizmos, aabb: &Aabb) {
    let min = aabb.min();
    let max = aabb.max();
    let center = (min + max) * 0.5;
    let size = max - min;

    // Draw the cuboid outline at the AABB center with the AABB size
    let outline_transform = Transform::from_translation(center.into()).with_scale(size.into());

    gizmos.cuboid(outline_transform, Color::srgb(1.0, 0.5, 0.0)); // Orange outline
}

/// Fallback outline for entities without proper bounds
fn draw_fallback_selection_box(gizmos: &mut Gizmos, global_transform: &GlobalTransform) {
    let translation = global_transform.translation();
    let default_size = Vec3::splat(1.0);

    let outline_transform = Transform::from_translation(translation).with_scale(default_size);

    gizmos.cuboid(outline_transform, Color::srgb(0.5, 0.5, 0.5)); // Gray outline
}

pub fn selection_box_system(
    show: Res<ShowSelectionBox>,
    selection: Res<EditorSelection>,
    mut gizmos: Gizmos,
    queries: SelectionBoxQueries,
    meshes: Res<Assets<Mesh>>,
) {
    if !show.0 {
        return;
    }

    for entity in selection.iter() {
        // Calculate the bounding box for the entity (including children)
        if let Some(world_aabb) = calculate_world_aabb(
            entity,
            &queries.mesh_query,
            &queries.sprite_query,
            &queries.aabb_query,
            &queries.children_query,
            &queries.transform_query,
            &meshes,
        ) {
            draw_selection_box(&mut gizmos, &world_aabb);
        } else {
            // Fallback to simple transform-based selection box
            if let Ok(global_transform) = queries.transform_query.get(entity) {
                draw_fallback_selection_box(&mut gizmos, global_transform);
            }
        }
    }
}

/// Calculate the world-space AABB for an entity and optionally its children
fn calculate_world_aabb(
    entity: Entity,
    mesh_query: &Query<(&GlobalTransform, &Mesh3d, Option<&Aabb>)>,
    sprite_query: &Query<(&GlobalTransform, &Sprite), Without<Mesh3d>>,
    aabb_query: &Query<(&GlobalTransform, &Aabb), (Without<Mesh3d>, Without<Sprite>)>,
    children_query: &Query<&Children>,
    transform_query: &Query<&GlobalTransform, (Without<Mesh3d>, Without<Sprite>, Without<Aabb>)>,
    meshes: &Assets<Mesh>,
) -> Option<Aabb> {
    let mut combined_aabb: Option<Aabb> = None;

    // Helper function to combine AABBs
    let mut combine_aabb = |new_aabb: Aabb| {
        if let Some(existing) = combined_aabb {
            combined_aabb = Some(combine_aabbs(&existing, &new_aabb));
        } else {
            combined_aabb = Some(new_aabb);
        }
    };

    // Try to get AABB from the entity itself
    if let Some(entity_aabb) = get_entity_aabb(
        entity,
        mesh_query,
        sprite_query,
        aabb_query,
        transform_query,
        meshes,
    ) {
        combine_aabb(entity_aabb);
    }

    // Recursively include children's AABBs
    if let Ok(children) = children_query.get(entity) {
        for &child in children {
            if let Some(child_aabb) = calculate_world_aabb(
                child,
                mesh_query,
                sprite_query,
                aabb_query,
                children_query,
                transform_query,
                meshes,
            ) {
                combine_aabb(child_aabb);
            }
        }
    }

    combined_aabb
}

/// Combine two AABBs into a single AABB that encompasses both
fn combine_aabbs(a: &Aabb, b: &Aabb) -> Aabb {
    let min = a.min().min(b.min());
    let max = a.max().max(b.max());
    Aabb::from_min_max(min.into(), max.into())
}

/// Get the AABB for a single entity
fn get_entity_aabb(
    entity: Entity,
    mesh_query: &Query<(&GlobalTransform, &Mesh3d, Option<&Aabb>)>,
    sprite_query: &Query<(&GlobalTransform, &Sprite), Without<Mesh3d>>,
    aabb_query: &Query<(&GlobalTransform, &Aabb), (Without<Mesh3d>, Without<Sprite>)>,
    transform_query: &Query<&GlobalTransform, (Without<Mesh3d>, Without<Sprite>, Without<Aabb>)>,
    meshes: &Assets<Mesh>,
) -> Option<Aabb> {
    // Try mesh entities first
    if let Ok((global_transform, mesh_handle, existing_aabb)) = mesh_query.get(entity) {
        // Use existing AABB if available, otherwise compute from mesh
        let local_aabb = if let Some(aabb) = existing_aabb {
            *aabb
        } else if let Some(_mesh) = meshes.get(&mesh_handle.0) {
            // TODO: Compute AABB from mesh if possible
            Aabb::from_min_max(Vec3::splat(-0.5), Vec3::splat(0.5))
        } else {
            return None;
        };

        return Some(transform_aabb(&local_aabb, global_transform));
    }

    // Try sprite entities
    if let Ok((global_transform, sprite)) = sprite_query.get(entity) {
        let size = sprite.custom_size.unwrap_or(Vec2::new(1.0, 1.0));
        let local_aabb = Aabb::from_min_max(
            Vec3::new(-size.x * 0.5, -size.y * 0.5, -0.01),
            Vec3::new(size.x * 0.5, size.y * 0.5, 0.01),
        );
        return Some(transform_aabb(&local_aabb, global_transform));
    }

    // Try entities with existing AABB components
    if let Ok((global_transform, aabb)) = aabb_query.get(entity) {
        return Some(transform_aabb(aabb, global_transform));
    }

    // Fallback for entities with just transforms
    if let Ok(global_transform) = transform_query.get(entity) {
        let default_size = 0.5;
        let local_aabb = Aabb::from_min_max(Vec3::splat(-default_size), Vec3::splat(default_size));
        return Some(transform_aabb(&local_aabb, global_transform));
    }

    None
}

/// Transform a local AABB to world space using `GlobalTransform`
fn transform_aabb(local_aabb: &Aabb, global_transform: &GlobalTransform) -> Aabb {
    // Get the 8 corners of the AABB
    let min = local_aabb.min();
    let max = local_aabb.max();
    let corners = [
        Vec3::new(min.x, min.y, min.z),
        Vec3::new(max.x, min.y, min.z),
        Vec3::new(min.x, max.y, min.z),
        Vec3::new(max.x, max.y, min.z),
        Vec3::new(min.x, min.y, max.z),
        Vec3::new(max.x, min.y, max.z),
        Vec3::new(min.x, max.y, max.z),
        Vec3::new(max.x, max.y, max.z),
    ]
    .map(|corner| global_transform.transform_point(corner));

    // Find the min/max of transformed corners
    let mut world_min = corners[0];
    let mut world_max = corners[0];

    for &corner in &corners[1..] {
        world_min = world_min.min(corner);
        world_max = world_max.max(corner);
    }

    Aabb::from_min_max(world_min, world_max)
}

pub fn spawn_selection_box_toggle_ui(mut commands: Commands) {
    info!("Spawning Selection Box Toggle UI");
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                right: Val::Px(20.0),
                width: Val::Px(100.0),
                height: Val::Px(15.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Show Selection Box"),
                TextFont::from_font_size(10.0),
                SelectionBoxToggleText,
            ));
        })
        .observe(
            |_trigger: On<Pointer<Click>>, mut show_selection: ResMut<ShowSelectionBox>| {
                show_selection.0 = !show_selection.0;
            },
        );
}

// System to update the button text when ShowSelectionBox changes
fn update_selection_box_toggle_text(
    show_selection: Res<ShowSelectionBox>,
    mut query: Query<&mut Text, With<SelectionBoxToggleText>>,
) {
    if show_selection.is_changed() {
        for mut text in &mut query {
            text.0 = if show_selection.0 {
                "Hide Selection Box".into()
            } else {
                "Show Selection Box".into()
            };
        }
    }
}
