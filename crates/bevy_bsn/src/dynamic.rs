use core::any::TypeId;
use std::any::Any;

use bevy::{
    platform_support::collections::hash_map::Entry,
    prelude::{AppTypeRegistry, Component, Mut, ReflectComponent},
    reflect::{PartialReflect, TypeRegistry},
    utils::TypeIdMap,
};
use variadics_please::all_tuples;

use crate::{
    Construct, ConstructContext, ConstructError, ConstructPatch, ConstructPatchExt, Key, Patch,
    ReflectConstruct, Scene,
};

/// Dynamic patch
pub trait DynamicPatch: Send + Sync + 'static {
    /// Layer this patch onto a [`DynamicScene`].
    fn dynamic_patch(&mut self, scene: &mut DynamicScene);

    /// Creates a new [`DynamicScene`], patches it, and pushes it as a child of the provided `parent_scene`.
    fn dynamic_patch_as_child(&mut self, parent_scene: &mut DynamicScene) {
        let mut child_scene = DynamicScene::default();
        self.dynamic_patch(&mut child_scene);
        parent_scene.push_child(child_scene);
    }

    /// Creates a new [`DynamicScene`], patches it, and returns it.
    fn dynamic_patch_as_new(&mut self) -> DynamicScene {
        let mut scene = DynamicScene::default();
        self.dynamic_patch(&mut scene);
        scene
    }
}

// Tuple impls
macro_rules! impl_patch_for_tuple {
    ($(#[$meta:meta])* $(($T:ident, $t:ident)),*) => {
        $(#[$meta])*
        impl<$($T: DynamicPatch),*> DynamicPatch for ($($T,)*) {
            fn dynamic_patch(&mut self, _scene: &mut DynamicScene) {
                let ($($t,)*) = self;
                $($t.dynamic_patch(_scene);)*
            }

            fn dynamic_patch_as_child(&mut self, _parent_scene: &mut DynamicScene) {
                let ($($t,)*) = self;
                $($t.dynamic_patch_as_child(_parent_scene);)*
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

impl<C, F> DynamicPatch for ConstructPatch<C, F>
where
    C: Construct + Component,
    F: FnMut(&mut C::Props) + Clone + Sync + Send + 'static,
{
    fn dynamic_patch(&mut self, scene: &mut DynamicScene) {
        scene.patch_typed::<C, F>(self.func.clone());
    }
}

/// Exists to allow either typed or reflect based patches in [`DynamicScene`].
trait DynamicComponentPatch: Send + Sync + 'static {
    fn patch_any(
        &mut self,
        type_id: TypeId,
        props: &mut dyn Any,
        registry: &TypeRegistry,
    ) -> Result<(), ConstructError>;
}

impl<C, F> DynamicComponentPatch for ConstructPatch<C, F>
where
    C: Construct + Component,
    F: FnMut(&mut C::Props) + Send + Sync + 'static,
{
    fn patch_any(
        &mut self,
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
    F: Fn(&mut dyn PartialReflect) + Send + Sync + 'static,
{
    fn patch_any(
        &mut self,
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
pub struct ComponentProps {
    type_id: TypeId,
    patches: Vec<Box<dyn DynamicComponentPatch>>,
    construct: DynamicConstructFn,
}

impl ComponentProps {
    /// Constructs the component using the patches and inserts it onto the context entity.
    pub fn construct(&mut self, context: &mut ConstructContext) -> Result<(), ConstructError> {
        match &self.construct {
            DynamicConstructFn::Typed(construct) => {
                construct(context, self.type_id, &mut self.patches)
            }
            DynamicConstructFn::Reflected => {
                construct_reflected_component(context, self.type_id, &mut self.patches)
            }
        }
    }
}

/// A dynamic scene containing dynamic patches and children.
#[derive(Default)]
pub struct DynamicScene {
    /// Maps component type ids to patches to be applied on the props before construction.
    component_props: TypeIdMap<ComponentProps>,
    /// Children of the scene.
    children: Vec<DynamicScene>,
    /// Optional key used for retaining.
    key: Option<Key>,
}

impl DynamicScene {
    /// Returns the component props of the dynamic scene.
    pub fn component_props(&mut self) -> &mut TypeIdMap<ComponentProps> {
        &mut self.component_props
    }

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

    /// Adds a typed component patch to the dynamic scene.
    pub fn patch_typed<C, F>(&mut self, patch: F)
    where
        C: Construct + Component,
        F: FnMut(&mut C::Props) + Sync + Send + 'static,
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

    /// Adds a reflected component patch to the dynamic scene.
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
    ///
    /// NOTE: This will drain the component patches and children of `self`, leaving it empty.
    fn dynamic_patch(&mut self, other: &mut DynamicScene) {
        // Push the component patches
        for (type_id, mut component_props) in self.component_props.drain() {
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
        other.children.extend(std::mem::take(&mut self.children));
    }
}

impl Scene for DynamicScene {
    fn root_count(&self) -> usize {
        1
    }

    fn construct(&mut self, context: &mut ConstructContext) -> Result<(), ConstructError> {
        // Construct and insert components
        for (_, component_props) in self.component_props.iter_mut() {
            component_props.construct(context)?;
        }

        // Spawn children
        for child in self.children.iter_mut() {
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
            &mut Vec<Box<dyn DynamicComponentPatch>>,
        ) -> Result<(), ConstructError>,
    ),
    Reflected,
}

fn construct_typed_component<C: Construct + Component>(
    context: &mut ConstructContext,
    type_id: TypeId,
    patches: &mut Vec<Box<dyn DynamicComponentPatch>>,
) -> Result<(), ConstructError> {
    let mut props = C::Props::default();
    {
        let registry = &context.world.resource::<AppTypeRegistry>().read();
        for patch in patches.iter_mut() {
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
    patches: &mut Vec<Box<dyn DynamicComponentPatch>>,
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
            for patch in patches.iter_mut() {
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
