use std::borrow::Cow;

use bevy::{
    asset::load_internal_asset,
    core_pipeline::core_3d::Transparent3d,
    ecs::{
        query::ROQueryItem,
        system::{
            lifetimeless::{Read, SRes},
            SystemParamItem,
        },
    },
    pbr::MeshPipelineKey,
    prelude::*,
    render::{
        mesh::PrimitiveTopology,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand,
            RenderCommandResult, SetItemPipeline, ViewSortedRenderPhases,
        },
        render_resource::{
            binding_types::uniform_buffer, BindGroup, BindGroupEntries, BindGroupLayout,
            BindGroupLayoutEntries, BlendState, ColorTargetState, ColorWrites, CompareFunction,
            DepthBiasState, DepthStencilState, DynamicUniformBuffer, FragmentState,
            MultisampleState, PipelineCache, PolygonMode, PrimitiveState, RenderPipelineDescriptor,
            ShaderStages, ShaderType, SpecializedRenderPipeline, SpecializedRenderPipelines,
            StencilFaceState, StencilState, TextureFormat, VertexState,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::BevyDefault,
        view::{ExtractedView, ViewTarget, VisibleEntities},
        Extract, ExtractSchedule, Render, RenderApp, RenderSet,
    },
};

use crate::InfiniteGridSettings;

const GRID_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(15204473893972682982);

pub fn render_app_builder(app: &mut App) {
    load_internal_asset!(app, GRID_SHADER_HANDLE, "grid.wgsl", Shader::from_wgsl);

    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    render_app
        .init_resource::<GridViewUniforms>()
        .init_resource::<InfiniteGridUniforms>()
        .init_resource::<GridDisplaySettingsUniforms>()
        .init_resource::<InfiniteGridPipeline>()
        .init_resource::<SpecializedRenderPipelines<InfiniteGridPipeline>>()
        .add_render_command::<Transparent3d, DrawInfiniteGrid>()
        .add_systems(
            ExtractSchedule,
            (extract_infinite_grids, extract_per_camera_settings),
        )
        .add_systems(
            Render,
            (prepare_infinite_grids, prepare_grid_view_uniforms)
                .in_set(RenderSet::PrepareResources),
        )
        .add_systems(
            Render,
            (
                prepare_bind_groups_for_infinite_grids,
                prepare_grid_view_bind_groups,
            )
                .in_set(RenderSet::PrepareBindGroups),
        )
        .add_systems(Render, queue_infinite_grids.in_set(RenderSet::Queue));
}

#[derive(Component)]
struct ExtractedInfiniteGrid {
    transform: GlobalTransform,
    grid: InfiniteGridSettings,
}

#[derive(Debug, ShaderType)]
pub struct InfiniteGridUniform {
    rot_matrix: Mat3,
    offset: Vec3,
    normal: Vec3,
}

#[derive(Debug, ShaderType)]
pub struct GridDisplaySettingsUniform {
    scale: f32,
    // 1 / fadeout_distance
    dist_fadeout_const: f32,
    dot_fadeout_const: f32,
    x_axis_color: Vec3,
    z_axis_color: Vec3,
    minor_line_color: Vec4,
    major_line_color: Vec4,
}

impl GridDisplaySettingsUniform {
    fn from_settings(settings: &InfiniteGridSettings) -> Self {
        Self {
            scale: settings.scale,
            dist_fadeout_const: 1. / settings.fadeout_distance,
            dot_fadeout_const: 1. / settings.dot_fadeout_strength,
            x_axis_color: settings.x_axis_color.to_linear().to_vec3(),
            z_axis_color: settings.z_axis_color.to_linear().to_vec3(),
            minor_line_color: settings.minor_line_color.to_linear().to_vec4(),
            major_line_color: settings.major_line_color.to_linear().to_vec4(),
        }
    }
}

#[derive(Resource, Default)]
struct InfiniteGridUniforms {
    uniforms: DynamicUniformBuffer<InfiniteGridUniform>,
}

#[derive(Resource, Default)]
struct GridDisplaySettingsUniforms {
    uniforms: DynamicUniformBuffer<GridDisplaySettingsUniform>,
}

#[derive(Component)]
struct InfiniteGridUniformOffsets {
    position_offset: u32,
    settings_offset: u32,
}

#[derive(Component)]
pub struct PerCameraSettingsUniformOffset {
    offset: u32,
}

#[derive(Resource)]
struct InfiniteGridBindGroup {
    value: BindGroup,
}

#[derive(Clone, ShaderType)]
pub struct GridViewUniform {
    projection: Mat4,
    inverse_projection: Mat4,
    view: Mat4,
    inverse_view: Mat4,
    world_position: Vec3,
}

#[derive(Resource, Default)]
pub struct GridViewUniforms {
    uniforms: DynamicUniformBuffer<GridViewUniform>,
}

#[derive(Component)]
pub struct GridViewUniformOffset {
    pub offset: u32,
}

#[derive(Component)]
struct GridViewBindGroup {
    value: BindGroup,
}

struct SetGridViewBindGroup<const I: usize>;

impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetGridViewBindGroup<I> {
    type Param = ();
    type ViewQuery = (Read<GridViewUniformOffset>, Read<GridViewBindGroup>);
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        _item: &P,
        (view_uniform, bind_group): ROQueryItem<'w, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
        _param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(I, &bind_group.value, &[view_uniform.offset]);
        RenderCommandResult::Success
    }
}

struct SetInfiniteGridBindGroup<const I: usize>;

impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetInfiniteGridBindGroup<I> {
    type Param = SRes<InfiniteGridBindGroup>;
    type ViewQuery = Option<Read<PerCameraSettingsUniformOffset>>;
    type ItemQuery = Read<InfiniteGridUniformOffsets>;

    #[inline]
    fn render<'w>(
        _item: &P,
        camera_settings_offset: ROQueryItem<'w, Self::ViewQuery>,
        base_offsets: Option<ROQueryItem<'w, Self::ItemQuery>>,
        bind_group: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(base_offsets) = base_offsets else {
            warn!("PerCameraSettingsUniformOffset missing");
            return RenderCommandResult::Failure;
        };
        pass.set_bind_group(
            I,
            &bind_group.into_inner().value,
            &[
                base_offsets.position_offset,
                camera_settings_offset
                    .map(|cs| cs.offset)
                    .unwrap_or(base_offsets.settings_offset),
            ],
        );
        RenderCommandResult::Success
    }
}

struct FinishDrawInfiniteGrid;

impl<P: PhaseItem> RenderCommand<P> for FinishDrawInfiniteGrid {
    type Param = ();
    type ViewQuery = ();
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
        _param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.draw(0..4, 0..1);
        RenderCommandResult::Success
    }
}

fn prepare_grid_view_uniforms(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut view_uniforms: ResMut<GridViewUniforms>,
    views: Query<(Entity, &ExtractedView)>,
) {
    view_uniforms.uniforms.clear();
    for (entity, camera) in views.iter() {
        let projection = camera.clip_from_view;
        let view = camera.world_from_view.compute_matrix();
        let inverse_view = view.inverse();
        commands.entity(entity).insert(GridViewUniformOffset {
            offset: view_uniforms.uniforms.push(&GridViewUniform {
                projection,
                view,
                inverse_view,
                inverse_projection: projection.inverse(),
                world_position: camera.world_from_view.translation(),
            }),
        });
    }

    view_uniforms
        .uniforms
        .write_buffer(&render_device, &render_queue)
}

fn prepare_grid_view_bind_groups(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    uniforms: Res<GridViewUniforms>,
    pipeline: Res<InfiniteGridPipeline>,
    views: Query<Entity, With<GridViewUniformOffset>>,
) {
    if let Some(binding) = uniforms.uniforms.binding() {
        for entity in views.iter() {
            let bind_group = render_device.create_bind_group(
                "grid-view-bind-group",
                &pipeline.view_layout,
                &BindGroupEntries::single(binding.clone()),
            );
            commands
                .entity(entity)
                .insert(GridViewBindGroup { value: bind_group });
        }
    }
}

fn extract_infinite_grids(
    mut commands: Commands,
    grids: Extract<
        Query<(
            Entity,
            &InfiniteGridSettings,
            &GlobalTransform,
            &VisibleEntities,
        )>,
    >,
) {
    let extracted: Vec<_> = grids
        .iter()
        .map(|(entity, grid, transform, visible_entities)| {
            (
                entity,
                (
                    ExtractedInfiniteGrid {
                        transform: *transform,
                        grid: *grid,
                    },
                    visible_entities.clone(),
                ),
            )
        })
        .collect();
    commands.insert_or_spawn_batch(extracted);
}

fn extract_per_camera_settings(
    mut commands: Commands,
    cameras: Extract<Query<(Entity, &InfiniteGridSettings), With<Camera>>>,
) {
    let extracted: Vec<_> = cameras
        .iter()
        .map(|(entity, settings)| (entity, *settings))
        .collect();
    commands.insert_or_spawn_batch(extracted);
}

fn prepare_infinite_grids(
    mut commands: Commands,
    grids: Query<(Entity, &ExtractedInfiniteGrid)>,
    cameras: Query<(Entity, &InfiniteGridSettings), With<ExtractedView>>,
    mut position_uniforms: ResMut<InfiniteGridUniforms>,
    mut settings_uniforms: ResMut<GridDisplaySettingsUniforms>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    position_uniforms.uniforms.clear();
    for (entity, extracted) in grids.iter() {
        let transform = extracted.transform;
        let t = transform.compute_transform();
        let offset = transform.translation();
        let normal = transform.up();
        let rot_matrix = Mat3::from_quat(t.rotation.inverse());
        commands.entity(entity).insert(InfiniteGridUniformOffsets {
            position_offset: position_uniforms.uniforms.push(&InfiniteGridUniform {
                rot_matrix,
                offset,
                normal: *normal,
            }),
            settings_offset: settings_uniforms
                .uniforms
                .push(&GridDisplaySettingsUniform::from_settings(&extracted.grid)),
        });
    }

    for (entity, settings) in cameras.iter() {
        commands
            .entity(entity)
            .insert(PerCameraSettingsUniformOffset {
                offset: settings_uniforms
                    .uniforms
                    .push(&GridDisplaySettingsUniform::from_settings(settings)),
            });
    }

    position_uniforms
        .uniforms
        .write_buffer(&render_device, &render_queue);

    settings_uniforms
        .uniforms
        .write_buffer(&render_device, &render_queue);
}

fn prepare_bind_groups_for_infinite_grids(
    mut commands: Commands,
    position_uniforms: Res<InfiniteGridUniforms>,
    settings_uniforms: Res<GridDisplaySettingsUniforms>,
    pipeline: Res<InfiniteGridPipeline>,
    render_device: Res<RenderDevice>,
) {
    let Some((position_binding, settings_binding)) = position_uniforms
        .uniforms
        .binding()
        .zip(settings_uniforms.uniforms.binding())
    else {
        return;
    };

    let bind_group = render_device.create_bind_group(
        "infinite-grid-bind-group",
        &pipeline.infinite_grid_layout,
        &BindGroupEntries::sequential((position_binding.clone(), settings_binding.clone())),
    );
    commands.insert_resource(InfiniteGridBindGroup { value: bind_group });
}

#[allow(clippy::too_many_arguments)]
fn queue_infinite_grids(
    pipeline_cache: Res<PipelineCache>,
    transparent_draw_functions: Res<DrawFunctions<Transparent3d>>,
    pipeline: Res<InfiniteGridPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<InfiniteGridPipeline>>,
    infinite_grids: Query<&ExtractedInfiniteGrid>,
    mut transparent_render_phases: ResMut<ViewSortedRenderPhases<Transparent3d>>,
    mut views: Query<(Entity, &VisibleEntities, &ExtractedView)>,
    msaa: Res<Msaa>,
) {
    let draw_function_id = transparent_draw_functions
        .read()
        .get_id::<DrawInfiniteGrid>()
        .unwrap();

    for (view_entity, entities, view) in views.iter_mut() {
        let Some(phase) = transparent_render_phases.get_mut(&view_entity) else {
            continue;
        };

        let mesh_key = MeshPipelineKey::from_hdr(view.hdr);
        let pipeline_id = pipelines.specialize(
            &pipeline_cache,
            &pipeline,
            GridPipelineKey {
                mesh_key,
                sample_count: msaa.samples(),
            },
        );
        for &entity in entities.iter::<With<InfiniteGridSettings>>() {
            if !infinite_grids
                .get(entity)
                .map(|grid| plane_check(&grid.transform, view.world_from_view.translation()))
                .unwrap_or(false)
            {
                continue;
            }
            phase.items.push(Transparent3d {
                pipeline: pipeline_id,
                entity,
                draw_function: draw_function_id,
                distance: f32::NEG_INFINITY,
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::NONE,
            });
        }
    }
}

fn plane_check(plane: &GlobalTransform, point: Vec3) -> bool {
    plane.up().dot(plane.translation() - point).abs() > f32::EPSILON
}

type DrawInfiniteGrid = (
    SetItemPipeline,
    SetGridViewBindGroup<0>,
    SetInfiniteGridBindGroup<1>,
    FinishDrawInfiniteGrid,
);

#[derive(Resource)]
struct InfiniteGridPipeline {
    view_layout: BindGroupLayout,
    infinite_grid_layout: BindGroupLayout,
}

impl FromWorld for InfiniteGridPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let view_layout = render_device.create_bind_group_layout(
            "grid-view-bind-group-layout",
            &BindGroupLayoutEntries::single(
                ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                uniform_buffer::<GridViewUniform>(true),
            ),
        );
        let infinite_grid_layout = render_device.create_bind_group_layout(
            "infinite-grid-bind-group-layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    uniform_buffer::<InfiniteGridUniform>(true),
                    uniform_buffer::<GridDisplaySettingsUniform>(true),
                ),
            ),
        );

        Self {
            view_layout,
            infinite_grid_layout,
        }
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct GridPipelineKey {
    mesh_key: MeshPipelineKey,
    sample_count: u32,
}

impl SpecializedRenderPipeline for InfiniteGridPipeline {
    type Key = GridPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let format = if key.mesh_key.contains(MeshPipelineKey::HDR) {
            ViewTarget::TEXTURE_FORMAT_HDR
        } else {
            TextureFormat::bevy_default()
        };

        RenderPipelineDescriptor {
            label: Some(Cow::Borrowed("grid-render-pipeline")),
            layout: vec![self.view_layout.clone(), self.infinite_grid_layout.clone()],
            push_constant_ranges: Vec::new(),
            vertex: VertexState {
                shader: GRID_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: Cow::Borrowed("vertex"),
                buffers: vec![],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: bevy::render::render_resource::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: CompareFunction::Greater,
                stencil: StencilState {
                    front: StencilFaceState::IGNORE,
                    back: StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            multisample: MultisampleState {
                count: key.sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                shader: GRID_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: Cow::Borrowed("fragment"),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
        }
    }
}
