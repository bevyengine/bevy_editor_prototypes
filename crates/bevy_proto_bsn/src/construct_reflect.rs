use std::any::{Any, TypeId};

use bevy::{
    app::App,
    reflect::{FromType, PartialReflect, Reflect, TypePath},
};

use crate::{Construct, ConstructContext, ConstructError};

/// A struct used to operate on reflected [`Construct`] trait of a type.
///
/// A [`ReflectConstruct`] for type `T` can be obtained via [`FromType::from_type`].
#[derive(Clone)]
pub struct ReflectConstruct {
    /// Construct an instance of this type.
    pub construct: fn(
        &mut ConstructContext,
        Box<dyn Reflect>,
    ) -> Result<Box<dyn PartialReflect>, ConstructError>,
    /// Get the default props for this type.
    pub default_props: fn() -> Box<dyn Reflect>,
    /// The type of props used to construct this type.
    pub props_type: TypeId,
    /// Downcasts an instance of `T::Props` to `&mut dyn`[`PartialReflect`].
    pub downcast_props_mut: fn(&mut dyn Any) -> Option<&mut dyn PartialReflect>,
}

impl ReflectConstruct {
    /// Constructs a value by calling `T::construct` with the given dynamic props.
    pub fn construct(
        &self,
        context: &mut ConstructContext,
        props: Box<dyn Reflect>,
    ) -> Result<Box<dyn PartialReflect>, ConstructError> {
        (self.construct)(context, props)
    }

    /// Returns the default props for this type.
    pub fn default_props(&self) -> Box<dyn Reflect> {
        (self.default_props)()
    }

    /// Downcasts an instance of `T::Props` to `&mut dyn`[`PartialReflect`].
    pub fn downcast_props_mut<'a>(
        &self,
        props: &'a mut dyn Any,
    ) -> Option<&'a mut dyn PartialReflect> {
        (self.downcast_props_mut)(props)
    }
}

impl<T: Construct + Reflect> FromType<T> for ReflectConstruct
where
    <T as Construct>::Props: Reflect + TypePath,
{
    fn from_type() -> Self {
        ReflectConstruct {
            construct: |context, props| {
                let Ok(props) = props.downcast::<T::Props>() else {
                    return Err(ConstructError::InvalidProps {
                        message: format!("failed to downcast props to {}", T::Props::type_path())
                            .into(),
                    });
                };
                let constructed = T::construct(context, *props)?;
                Ok(Box::new(constructed))
            },
            default_props: || Box::new(T::Props::default()),
            props_type: TypeId::of::<T::Props>(),
            downcast_props_mut: |props| {
                props
                    .downcast_mut::<T::Props>()
                    .map(PartialReflect::as_partial_reflect_mut)
            },
        }
    }
}

/// Workaround for blanket-implemented types upstream. Should not be needed when [`Construct`]/[`ReflectConstruct`] is in main.
pub(crate) fn register_reflect_construct(app: &mut App) {
    app.register_type_data::<bevy::animation::AnimationPlayer, ReflectConstruct>();
    app.register_type_data::<bevy::animation::graph::AnimationGraphHandle, ReflectConstruct>();
    app.register_type_data::<bevy::animation::transition::AnimationTransitions, ReflectConstruct>();

    app.register_type_data::<bevy::audio::PlaybackSettings, ReflectConstruct>();
    app.register_type_data::<bevy::audio::SpatialListener, ReflectConstruct>();

    app.register_type_data::<bevy::anti_alias::contrast_adaptive_sharpening::ContrastAdaptiveSharpening, ReflectConstruct>();
    app.register_type_data::<bevy::anti_alias::contrast_adaptive_sharpening::DenoiseCas, ReflectConstruct>();
    app.register_type_data::<bevy::anti_alias::fxaa::Fxaa, ReflectConstruct>();
    app.register_type_data::<bevy::anti_alias::smaa::Smaa, ReflectConstruct>();
    app.register_type_data::<bevy::anti_alias::taa::TemporalAntiAliasing, ReflectConstruct>();

    app.register_type_data::<bevy::camera::Camera, ReflectConstruct>();
    app.register_type_data::<bevy::camera::Camera2d, ReflectConstruct>();
    app.register_type_data::<bevy::camera::Camera3d, ReflectConstruct>();
    app.register_type_data::<bevy::camera::CameraMainTextureUsages, ReflectConstruct>();
    app.register_type_data::<bevy::camera::Exposure, ReflectConstruct>();
    app.register_type_data::<bevy::camera::ManualTextureViewHandle, ReflectConstruct>();
    app.register_type_data::<bevy::camera::CustomProjection, ReflectConstruct>();
    app.register_type_data::<bevy::camera::visibility::Visibility, ReflectConstruct>();
    app.register_type_data::<bevy::camera::visibility::InheritedVisibility, ReflectConstruct>();
    app.register_type_data::<bevy::camera::visibility::VisibilityRange, ReflectConstruct>();
    app.register_type_data::<bevy::camera::visibility::RenderLayers, ReflectConstruct>();
    app.register_type_data::<bevy::camera::visibility::ViewVisibility, ReflectConstruct>();
    app.register_type_data::<bevy::camera::visibility::VisibilityClass, ReflectConstruct>();
    app.register_type_data::<bevy::camera::visibility::VisibleEntities, ReflectConstruct>();
    app.register_type_data::<bevy::camera::visibility::CascadesVisibleEntities, ReflectConstruct>();
    app.register_type_data::<bevy::camera::visibility::CubemapVisibleEntities, ReflectConstruct>();
    app.register_type_data::<bevy::camera::visibility::VisibleMeshEntities, ReflectConstruct>();
    app.register_type_data::<bevy::camera::primitives::Aabb, ReflectConstruct>();
    app.register_type_data::<bevy::camera::primitives::CascadesFrusta, ReflectConstruct>();
    app.register_type_data::<bevy::camera::primitives::CubemapFrusta, ReflectConstruct>();
    app.register_type_data::<bevy::camera::primitives::Frustum, ReflectConstruct>();

    app.register_type_data::<bevy::core_pipeline::oit::OrderIndependentTransparencySettings, ReflectConstruct>();
    app.register_type_data::<bevy::core_pipeline::prepass::DepthPrepass, ReflectConstruct>();
    app.register_type_data::<bevy::core_pipeline::prepass::MotionVectorPrepass, ReflectConstruct>();
    app.register_type_data::<bevy::core_pipeline::prepass::NormalPrepass, ReflectConstruct>();
    app.register_type_data::<bevy::core_pipeline::Skybox, ReflectConstruct>();

    app.register_type_data::<bevy::ecs::name::Name, ReflectConstruct>();

    app.register_type_data::<bevy::gizmos::retained::Gizmo, ReflectConstruct>();

    app.register_type_data::<bevy::gltf::GltfExtras, ReflectConstruct>();
    app.register_type_data::<bevy::gltf::GltfMaterialExtras, ReflectConstruct>();
    app.register_type_data::<bevy::gltf::GltfMaterialName, ReflectConstruct>();
    app.register_type_data::<bevy::gltf::GltfMeshExtras, ReflectConstruct>();
    app.register_type_data::<bevy::gltf::GltfSceneExtras, ReflectConstruct>();

    app.register_type_data::<bevy::input_focus::AutoFocus, ReflectConstruct>();
    app.register_type_data::<bevy::input_focus::tab_navigation::TabGroup, ReflectConstruct>();
    app.register_type_data::<bevy::input_focus::tab_navigation::TabIndex, ReflectConstruct>();

    app.register_type_data::<bevy::input::gamepad::GamepadSettings, ReflectConstruct>();

    app.register_type_data::<bevy::light::LightProbe, ReflectConstruct>();
    app.register_type_data::<bevy::light::AmbientLight, ReflectConstruct>();
    app.register_type_data::<bevy::light::Cascades, ReflectConstruct>();
    app.register_type_data::<bevy::light::CascadeShadowConfig, ReflectConstruct>();
    app.register_type_data::<bevy::light::DirectionalLight, ReflectConstruct>();
    app.register_type_data::<bevy::light::PointLight, ReflectConstruct>();
    app.register_type_data::<bevy::light::SpotLight, ReflectConstruct>();
    app.register_type_data::<bevy::light::IrradianceVolume, ReflectConstruct>();
    app.register_type_data::<bevy::light::FogVolume, ReflectConstruct>();
    app.register_type_data::<bevy::light::VolumetricFog, ReflectConstruct>();
    app.register_type_data::<bevy::light::VolumetricLight, ReflectConstruct>();

    app.register_type_data::<bevy::pbr::Atmosphere, ReflectConstruct>();
    app.register_type_data::<bevy::pbr::AtmosphereSettings, ReflectConstruct>();
    app.register_type_data::<bevy::pbr::RenderCascadesVisibleEntities, ReflectConstruct>();
    app.register_type_data::<bevy::pbr::RenderCubemapVisibleEntities, ReflectConstruct>();
    app.register_type_data::<bevy::pbr::RenderVisibleMeshEntities, ReflectConstruct>();
    app.register_type_data::<bevy::pbr::DistanceFog, ReflectConstruct>();
    app.register_type_data::<bevy::prelude::EnvironmentMapLight, ReflectConstruct>();
    app.register_type_data::<bevy::pbr::Lightmap, ReflectConstruct>();
    //app.register_type_data::<bevy::pbr::MeshMaterial3d, ReflectConstruct>();
    app.register_type_data::<bevy::pbr::ScreenSpaceAmbientOcclusion, ReflectConstruct>();
    app.register_type_data::<bevy::pbr::ScreenSpaceReflections, ReflectConstruct>();
    app.register_type_data::<bevy::pbr::wireframe::NoWireframe, ReflectConstruct>();
    app.register_type_data::<bevy::pbr::wireframe::Wireframe, ReflectConstruct>();
    app.register_type_data::<bevy::pbr::wireframe::WireframeColor, ReflectConstruct>();

    app.register_type_data::<bevy::post_process::auto_exposure::AutoExposure, ReflectConstruct>();
    app.register_type_data::<bevy::post_process::bloom::Bloom, ReflectConstruct>();
    app.register_type_data::<bevy::post_process::dof::DepthOfField, ReflectConstruct>();
    app.register_type_data::<bevy::post_process::effect_stack::ChromaticAberration, ReflectConstruct>();
    app.register_type_data::<bevy::post_process::motion_blur::MotionBlur, ReflectConstruct>();

    app.register_type_data::<bevy::picking::mesh_picking::ray_cast::RayCastBackfaces, ReflectConstruct>();
    app.register_type_data::<bevy::picking::mesh_picking::MeshPickingCamera, ReflectConstruct>();
    app.register_type_data::<bevy::picking::Pickable, ReflectConstruct>();
    app.register_type_data::<bevy::picking::pointer::PointerInteraction, ReflectConstruct>();
    app.register_type_data::<bevy::picking::pointer::PointerLocation, ReflectConstruct>();
    app.register_type_data::<bevy::picking::pointer::PointerPress, ReflectConstruct>();

    app.register_type_data::<bevy::render::camera::TemporalJitter, ReflectConstruct>();
    app.register_type_data::<bevy::render::experimental::occlusion_culling::OcclusionCulling, ReflectConstruct>();
    app.register_type_data::<bevy::render::sync_world::SyncToRenderWorld, ReflectConstruct>();
    app.register_type_data::<bevy::render::sync_world::TemporaryRenderEntity, ReflectConstruct>();
    app.register_type_data::<bevy::render::view::ColorGrading, ReflectConstruct>();
    app.register_type_data::<bevy::render::view::visibility::RenderVisibleEntities, ReflectConstruct>();

    app.register_type_data::<bevy::scene::DynamicSceneRoot, ReflectConstruct>();
    app.register_type_data::<bevy::scene::SceneRoot, ReflectConstruct>();

    //app.register_type_data::<bevy::sprite::MeshMaterial2d, ReflectConstruct>();
    app.register_type_data::<bevy::sprite::SpritePickingCamera, ReflectConstruct>();
    app.register_type_data::<bevy::sprite::Sprite, ReflectConstruct>();
    app.register_type_data::<bevy::sprite::Text2d, ReflectConstruct>();

    app.register_type_data::<bevy::sprite_render::NoWireframe2d, ReflectConstruct>();
    app.register_type_data::<bevy::sprite_render::Wireframe2d, ReflectConstruct>();
    app.register_type_data::<bevy::sprite_render::Wireframe2dColor, ReflectConstruct>();

    app.register_type_data::<bevy::mesh::Mesh2d, ReflectConstruct>();
    app.register_type_data::<bevy::mesh::Mesh3d, ReflectConstruct>();
    app.register_type_data::<bevy::mesh::MeshTag, ReflectConstruct>();
    app.register_type_data::<bevy::mesh::morph::MeshMorphWeights, ReflectConstruct>();
    app.register_type_data::<bevy::mesh::morph::MorphWeights, ReflectConstruct>();
    app.register_type_data::<bevy::mesh::skinning::SkinnedMesh, ReflectConstruct>();

    // app.register_type_data::<bevy::prelude::StateScoped, ReflectConstruct>();

    app.register_type_data::<bevy::text::TextBounds, ReflectConstruct>();
    app.register_type_data::<bevy::text::TextLayoutInfo, ReflectConstruct>();
    app.register_type_data::<bevy::text::ComputedTextBlock, ReflectConstruct>();
    app.register_type_data::<bevy::text::TextColor, ReflectConstruct>();
    app.register_type_data::<bevy::text::TextFont, ReflectConstruct>();
    app.register_type_data::<bevy::text::TextLayout, ReflectConstruct>();
    app.register_type_data::<bevy::text::TextSpan, ReflectConstruct>();

    app.register_type_data::<bevy::transform::components::GlobalTransform, ReflectConstruct>();
    app.register_type_data::<bevy::transform::components::Transform, ReflectConstruct>();

    // app.register_type_data::<bevy::ui::experimental::ghost_hierarchy::GhostNode, ReflectConstruct>();
    app.register_type_data::<bevy::ui::RelativeCursorPosition, ReflectConstruct>();
    // app.register_type_data::<bevy::ui::MaterialNode, ReflectConstruct>();
    app.register_type_data::<bevy::ui::BackgroundColor, ReflectConstruct>();
    app.register_type_data::<bevy::ui::BorderColor, ReflectConstruct>();
    app.register_type_data::<bevy::ui::BorderRadius, ReflectConstruct>();
    app.register_type_data::<bevy::ui::BoxShadow, ReflectConstruct>();
    app.register_type_data::<bevy::ui_render::BoxShadowSamples, ReflectConstruct>();
    app.register_type_data::<bevy::ui::CalculatedClip, ReflectConstruct>();
    app.register_type_data::<bevy::ui::ComputedNode, ReflectConstruct>();
    app.register_type_data::<bevy::ui::ComputedUiTargetCamera, ReflectConstruct>();
    app.register_type_data::<bevy::ui::GlobalZIndex, ReflectConstruct>();
    app.register_type_data::<bevy::ui::LayoutConfig, ReflectConstruct>();
    app.register_type_data::<bevy::ui::Node, ReflectConstruct>();
    app.register_type_data::<bevy::ui::Outline, ReflectConstruct>();
    app.register_type_data::<bevy::ui::ScrollPosition, ReflectConstruct>();
    app.register_type_data::<bevy::ui::widget::TextShadow, ReflectConstruct>();
    app.register_type_data::<bevy::ui::ZIndex, ReflectConstruct>();
    app.register_type_data::<bevy::ui::widget::Button, ReflectConstruct>();
    app.register_type_data::<bevy::ui::widget::ImageNode, ReflectConstruct>();
    app.register_type_data::<bevy::ui::widget::ImageNodeSize, ReflectConstruct>();
    app.register_type_data::<bevy::ui::widget::Label, ReflectConstruct>();
    app.register_type_data::<bevy::ui::widget::Text, ReflectConstruct>();
    app.register_type_data::<bevy::ui::widget::TextNodeFlags, ReflectConstruct>();

    app.register_type_data::<bevy::window::PrimaryWindow, ReflectConstruct>();
    app.register_type_data::<bevy::window::Window, ReflectConstruct>();
}
