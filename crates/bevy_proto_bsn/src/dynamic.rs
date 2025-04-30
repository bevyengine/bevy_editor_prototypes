use core::any::TypeId;
use std::any::Any;

use bevy::{
    platform::collections::hash_map::Entry,
    prelude::{AppTypeRegistry, Component, Mut, ReflectComponent},
    reflect::{PartialReflect, TypeRegistry},
    utils::TypeIdMap,
};
use variadics_please::all_tuples;

use crate::{
    Construct, ConstructContext, ConstructError, ConstructPatch, ConstructPatchExt, Key,
    ReflectConstruct, Scene,
};

/// Trait for entity patches that can be applied to a [`DynamicScene`].
///
/// Implemented for tuples of [`DynamicPatch`] and [`Vec<impl DynamicPatch>`], allowing multiple patches to be applied at once, either:
/// - Onto the same entity, to power things like inheritance, using [`DynamicPatch::dynamic_patch`].
/// - As children using [`DynamicPatch::dynamic_patch_as_children`]
pub trait DynamicPatch: Sized + Send + Sync + 'static {
    /// Apply this patch to the provided [`DynamicScene`].
    fn dynamic_patch(self, scene: &mut DynamicScene);

    /// Creates a new [`DynamicScene`], patches it with `self`, and returns it.
    fn into_dynamic_scene(self) -> DynamicScene {
        let mut scene = DynamicScene::default();
        self.dynamic_patch(&mut scene);
        scene
    }

    /// Creates a new [`DynamicScene`] per root in `self`, patches it, and pushes it as a child of the provided `parent_scene`.
    fn dynamic_patch_as_children(self, parent_scene: &mut DynamicScene) {
        let child_scene = self.into_dynamic_scene();
        parent_scene.push_child(child_scene);
    }
}

// Tuple impls
macro_rules! impl_patch_for_tuple {
    ($(#[$meta:meta])* $(($T:ident, $t:ident)),*) => {
        $(#[$meta])*
        impl<$($T: DynamicPatch),*> DynamicPatch for ($($T,)*) {
            fn dynamic_patch(self, _scene: &mut DynamicScene) {
                let ($($t,)*) = self;
                $($t.dynamic_patch(_scene);)*
            }

            fn dynamic_patch_as_children(self, _parent_scene: &mut DynamicScene) {
                let ($($t,)*) = self;
                $($t.dynamic_patch_as_children(_parent_scene);)*
            }
        }
    };
}

all_tuples!(
    #[doc(fake_variadic)]
    impl_patch_for_tuple,
    0,
    12,
    T,
    t
);

impl<D: DynamicPatch> DynamicPatch for Vec<D> {
    fn dynamic_patch(self, dynamic_scene: &mut DynamicScene) {
        for scene in self {
            scene.dynamic_patch(dynamic_scene);
        }
    }

    fn dynamic_patch_as_children(self, parent_scene: &mut DynamicScene) {
        for scene in self {
            scene.dynamic_patch_as_children(parent_scene);
        }
    }
}

impl<C, F> DynamicPatch for ConstructPatch<C, F>
where
    C: Construct + Component,
    F: FnOnce(&mut C::Props) + Clone + Sync + Send + 'static,
{
    fn dynamic_patch(self, scene: &mut DynamicScene) {
        scene.patch_typed::<C, F>(self.func);
    }
}

/// Exists to allow either typed or reflect based patches in [`DynamicScene`].
trait DynamicComponentPatch: Send + Sync + 'static {
    fn patch_any(
        self: Box<Self>,
        type_id: TypeId,
        props: &mut dyn Any,
        registry: &TypeRegistry,
    ) -> Result<(), ConstructError>;
}

impl<C, F> DynamicComponentPatch for ConstructPatch<C, F>
where
    C: Construct + Component,
    F: FnOnce(&mut C::Props) + Send + Sync + 'static,
{
    fn patch_any(
        self: Box<Self>,
        _: TypeId,
        props: &mut dyn Any,
        _: &TypeRegistry,
    ) -> Result<(), ConstructError> {
        let props = props
            .downcast_mut::<C::Props>()
            .ok_or(ConstructError::InvalidProps {
                message: "failed to downcast props".into(),
            })?;
        (self.func)(props);
        Ok(())
    }
}

impl<F> DynamicComponentPatch for F
where
    F: FnOnce(&mut dyn PartialReflect) + Send + Sync + 'static,
{
    fn patch_any(
        self: Box<Self>,
        type_id: TypeId,
        props: &mut dyn Any,
        registry: &TypeRegistry,
    ) -> Result<(), ConstructError> {
        let t = registry
            .get(type_id)
            .ok_or(ConstructError::Custom("missing type in registry"))?;
        let reflect_construct = t.data::<ReflectConstruct>().ok_or(ConstructError::Custom(
            "No registered ReflectConstruct for component",
        ))?;

        let props =
            reflect_construct
                .downcast_props_mut(props)
                .ok_or(ConstructError::InvalidProps {
                    message: "failed to downcast props".into(),
                })?;

        (self)(props);

        Ok(())
    }
}

/// Holds component patches and a dynamic construct function.
pub(crate) struct ComponentProps {
    type_id: TypeId,
    patches: Vec<Box<dyn DynamicComponentPatch>>,
    construct: DynamicConstructFn,
}

impl ComponentProps {
    /// Constructs the component using the patches and inserts it onto the context entity.
    pub fn construct(self, context: &mut ConstructContext) -> Result<(), ConstructError> {
        match &self.construct {
            DynamicConstructFn::Typed(construct) => construct(context, self.type_id, self.patches),
            DynamicConstructFn::Reflected => {
                construct_reflected_component(context, self.type_id, self.patches)
            }
        }
    }
}

/// A dynamic scene containing dynamic patches and children.
#[derive(Default)]
pub struct DynamicScene {
    /// Maps component type ids to patches to be applied on the props before construction.
    pub(crate) component_props: TypeIdMap<ComponentProps>,
    /// Children of the scene.
    pub(crate) children: Vec<DynamicScene>,
    /// Optional key used for retaining.
    key: Option<Key>,
}

impl DynamicScene {
    /// Returns the optional key of the root entity in the dynamic scene.
    pub fn key(&self) -> &Option<Key> {
        &self.key
    }

    /// Returns a mutable borrow of the children of the dynamic scene.
    pub fn children_mut(&mut self) -> &mut Vec<DynamicScene> {
        &mut self.children
    }

    /// Add a child to the dynamic scene.
    pub fn push_child(&mut self, child: DynamicScene) {
        self.children.push(child);
    }

    /// Adds a typed component patch to the root entity of this dynamic scene.
    pub fn patch_typed<C, F>(&mut self, patch: F)
    where
        C: Construct + Component,
        F: FnOnce(&mut C::Props) + Clone + Sync + Send + 'static,
    {
        let construct = DynamicConstructFn::Typed(|context, type_id, patches| {
            construct_typed_component::<C>(context, type_id, patches)
        });

        match self.component_props.entry(TypeId::of::<C>()) {
            Entry::Vacant(entry) => {
                entry.insert(ComponentProps {
                    type_id: TypeId::of::<C>(),
                    patches: vec![Box::new(C::patch(patch))],
                    construct,
                });
            }
            Entry::Occupied(mut entry) => {
                let entry = entry.get_mut();
                if matches!(entry.construct, DynamicConstructFn::Reflected) {
                    entry.construct = construct;
                }
                entry.patches.push(Box::new(C::patch(patch)));
            }
        }
    }

    /// Adds a reflected component patch to the root entity of this dynamic scene.
    pub fn patch_reflected<F>(&mut self, type_id: TypeId, patch: F)
    where
        F: Fn(&mut dyn PartialReflect) + Send + Sync + 'static,
    {
        match self.component_props.entry(type_id) {
            Entry::Vacant(entry) => {
                entry.insert(ComponentProps {
                    type_id,
                    patches: vec![Box::new(patch)],
                    construct: DynamicConstructFn::Reflected,
                });
            }
            Entry::Occupied(mut entry) => {
                entry.get_mut().patches.push(Box::new(patch));
            }
        }
    }
}

impl DynamicPatch for DynamicScene {
    /// Dynamic patch this scene onto another [`DynamicScene`].
    fn dynamic_patch(self, other: &mut DynamicScene) {
        // Push the component patches
        for (type_id, mut component_props) in self.component_props {
            other
                .component_props
                .entry(type_id)
                .and_modify(|props| {
                    props
                        .patches
                        .extend(std::mem::take(&mut component_props.patches));
                })
                .or_insert(component_props);
        }

        // Push the children
        other.children.extend(self.children);
    }
}

impl Scene for DynamicScene {
    fn root_count(&self) -> usize {
        1
    }

    fn construct(self, context: &mut ConstructContext) -> Result<(), ConstructError> {
        // Construct and insert components
        for (_, component_props) in self.component_props {
            component_props.construct(context)?;
        }

        // Spawn children
        for child in self.children {
            let child_id = context.world.spawn_empty().id();
            context.world.entity_mut(context.id).add_child(child_id);
            child.construct(&mut ConstructContext {
                id: child_id,
                world: context.world,
            })?;
        }

        Ok(())
    }
}

enum DynamicConstructFn {
    Typed(
        fn(
            &mut ConstructContext,
            TypeId,
            Vec<Box<dyn DynamicComponentPatch>>,
        ) -> Result<(), ConstructError>,
    ),
    Reflected,
}

fn construct_typed_component<C: Construct + Component>(
    context: &mut ConstructContext,
    type_id: TypeId,
    patches: Vec<Box<dyn DynamicComponentPatch>>,
) -> Result<(), ConstructError> {
    let mut props = C::Props::default();
    {
        let registry = &context.world.resource::<AppTypeRegistry>().read();
        for patch in patches {
            patch.patch_any(type_id, &mut props, registry)?;
        }
    }
    let component = C::construct(context, props)?;
    context.world.entity_mut(context.id).insert(component);
    Ok(())
}

fn construct_reflected_component(
    context: &mut ConstructContext,
    type_id: TypeId,
    patches: Vec<Box<dyn DynamicComponentPatch>>,
) -> Result<(), ConstructError> {
    context
        .world
        .resource_scope(|world, app_registry: Mut<AppTypeRegistry>| {
            let registry = app_registry.read();
            let t = registry
                .get(type_id)
                .ok_or(ConstructError::Custom("missing type in registry"))?;
            let reflect_construct = t.data::<ReflectConstruct>().ok_or(ConstructError::Custom(
                "No registered ReflectConstruct for component",
            ))?;

            let reflect_component = t.data::<ReflectComponent>().ok_or(ConstructError::Custom(
                "No registered ReflectComponent for component",
            ))?;

            // Prepare props
            let mut props = reflect_construct.default_props();
            for patch in patches {
                patch.patch_any(type_id, props.as_any_mut(), &registry)?;
            }

            // Construct component
            let component = reflect_construct
                .construct(&mut ConstructContext::new(context.id, world), props)?;

            // Insert component on entity
            let mut entity = world.entity_mut(context.id);
            reflect_component.insert(&mut entity, component.as_ref(), &registry);

            Ok(())
        })
}
