use core::any::TypeId;

use bevy::{
    asset::Asset,
    log::{error, warn},
    prelude::{AppTypeRegistry, Component, Mut, ReflectComponent},
    reflect::{PartialReflect, Reflect, TypePath},
    utils::TypeIdMap,
};
use variadics_please::all_tuples;

use crate::{Construct, ConstructContext, ConstructError, ConstructPatch, ReflectConstruct};

/// Dynamic patch
pub trait DynamicPatch: Send + Sync + 'static {
    /// Adds this patch "on top" of the dynamic scene by updating the dynamic props.
    fn dynamic_patch(&mut self, scene: &mut DynamicScene);
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
    C: Construct + Component + PartialReflect,
    C::Props: Reflect,
    F: Fn(&mut C::Props) + Clone + Sync + Send + 'static,
{
    fn dynamic_patch(&mut self, scene: &mut DynamicScene) {
        let patches = scene.component_props.entry(TypeId::of::<C>()).or_default();

        let func = self.func.clone();

        patches.push(Box::new(move |props: &mut dyn Reflect| {
            (func)(props.downcast_mut::<C::Props>().unwrap());
        }));
    }
}

/// Trait implemented for functions that can patch [`Reflect`] props.
pub trait ReflectPatch: Sync + Send {
    /// Patch the given props.
    fn patch(&self, props: &mut dyn Reflect);
}

impl<F> ReflectPatch for F
where
    F: Fn(&mut dyn Reflect) + Sync + Send,
{
    fn patch(&self, props: &mut dyn Reflect) {
        (self)(props);
    }
}

/// A dynamic scene containing dynamic patches and children.
#[derive(Default, Asset, TypePath)]
pub struct DynamicScene {
    /// Maps component type ids to patches to be applied on the props before construction.
    pub component_props: TypeIdMap<Vec<Box<dyn ReflectPatch>>>,
    /// Children of the scene.
    pub children: Vec<DynamicScene>,
}

impl DynamicScene {
    /// Constructs this dynamic scene onto the context entity by:
    ///  - Constructing and inserting the resulting components.
    ///  - Spawning children recursively.
    pub fn construct(&self, context: &mut ConstructContext) -> Result<(), ConstructError> {
        self.construct_components(context)?;
        self.construct_children(context)?;
        Ok(())
    }

    /// Construct and insert the dynamic components onto the context entity.
    pub fn construct_components(
        &self,
        context: &mut ConstructContext,
    ) -> Result<(), ConstructError> {
        // Construct components
        for (type_id, patches) in self.component_props.iter() {
            context
                .world
                .resource_scope(|world, app_registry: Mut<AppTypeRegistry>| {
                    let registry = app_registry.read();
                    let t = registry
                        .get(*type_id)
                        .expect("failed to get type from registry");
                    let Some(reflect_construct) = t.data::<ReflectConstruct>() else {
                        warn!(
                            "No registered ReflectConstruct for component: {:?}. Skipping construction. Consider adding #[reflect(Construct)].",
                            t.type_info().type_path()
                        );
                        return;
                    };
                    let Some(reflect_component) = t.data::<ReflectComponent>() else {
                        warn!(
                            "No registered ReflectComponent for component: {:?}. Skipping construction. Consider adding #[reflect(Component)].",
                            t.type_info().type_path()
                        );
                        return;
                    };

                    // Prepare props
                    let mut props = reflect_construct.default_props();
                    for patch in patches.iter() {
                        patch.patch(props.as_mut());
                    }

                    // Construct component
                    match reflect_construct.construct(
                        &mut ConstructContext {
                            id: context.id,
                            world,
                        },
                        props,
                    ) {
                        Ok(component) => {
                            // Insert component on entity
                            let mut entity = world.entity_mut(context.id);
                            reflect_component.insert(&mut entity, component.as_ref(), &registry);
                        }
                        Err(e) => {
                            error!(
                                "failed to construct component '{:?}': {}",
                                t.type_info().type_path(), e
                            );
                        }
                    }
                });
        }

        Ok(())
    }

    /// Construct and spawn the dynamic children under the context entity.
    pub fn construct_children(&self, context: &mut ConstructContext) -> Result<(), ConstructError> {
        // Spawn children
        for child in self.children.iter() {
            let child_id = context.world.spawn_empty().id();
            context.world.entity_mut(context.id).add_child(child_id);
            child.construct(&mut ConstructContext {
                id: child_id,
                world: context.world,
            })?;
        }

        Ok(())
    }

    /// Add a child to the dynamic scene.
    pub fn push_child(&mut self, child: DynamicScene) {
        self.children.push(child);
    }
}
