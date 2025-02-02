use bevy::prelude::{
    ChildSpawnerCommands, Commands, EntityCommand, EntityCommands, EntityWorldMut,
};
use variadics_please::all_tuples_with_size;

use crate::{
    ConstructContext, ConstructContextPatchExt, ConstructError, DynamicPatch, DynamicScene, Key,
    Patch,
};

/// Destination trait for [`EntityPatch`].
pub trait Scene {
    /// The number of root entities in this scene.
    const ROOT_COUNT: usize;

    /// Constructs a [`Scene`], inserts the components to the context entity, and recursively spawns scene descendants.
    ///
    /// If this is called on a multi-root scene, each root entity will be constructed separately.
    fn construct(self, context: &mut ConstructContext) -> Result<(), ConstructError>;

    /// Constructs and spawns a [`Scene`] as a child (or children if multi-root) under the context entity recursively.
    fn spawn(self, context: &mut ConstructContext) -> Result<(), ConstructError>;

    /// Dynamically applies the patches of this scene to a [`DynamicScene`], effectively overwriting any patched props.
    fn dynamic_patch(&mut self, scene: &mut DynamicScene);

    /// Dynamically patches the scene and pushes it as a child of the [`DynamicScene`].
    fn dynamic_patch_as_child(&mut self, scene: &mut DynamicScene);
}

impl Scene for () {
    const ROOT_COUNT: usize = 0;

    fn construct(self, _: &mut ConstructContext) -> Result<(), ConstructError> {
        Ok(())
    }

    fn spawn(self, _: &mut ConstructContext) -> Result<(), ConstructError> {
        Ok(())
    }

    fn dynamic_patch(&mut self, _: &mut DynamicScene) {}

    fn dynamic_patch_as_child(&mut self, _: &mut DynamicScene) {}
}

// Tuple impls
macro_rules! impl_scene_tuple {
    ($N:expr, $(#[$meta:meta])* $(($S:ident, $s:ident)),*) => {
        $(#[$meta])*
        impl<$($S: Scene),*> Scene for ($($S,)*)
        {
            const ROOT_COUNT: usize = $N;

            fn construct(self, context: &mut ConstructContext) -> Result<(), ConstructError> {
                let ($($s,)*) = self;
                $($s.construct(context)?;)*
                Ok(())
            }

            fn spawn(self, context: &mut ConstructContext) -> Result<(), ConstructError> {
                let ($($s,)*) = self;
                $($s.spawn(context)?;)*
                Ok(())
            }

            fn dynamic_patch(
                &mut self,
                scene: &mut DynamicScene,
            ) {
                let ($($s,)*) = self;
                $($s.dynamic_patch(scene);)*
            }

            fn dynamic_patch_as_child(&mut self, scene: &mut DynamicScene) {
                let ($($s,)*) = self;
                $($s.dynamic_patch_as_child(scene);)*
            }
        }
    };
}

all_tuples_with_size!(
    #[doc(fake_variadic)]
    impl_scene_tuple,
    1,
    12,
    S,
    s
);

/// Represents a tree of entities and patches to be applied to them.
pub struct EntityPatch<I, P, C>
where
    I: Scene,
    P: Patch + DynamicPatch,
    C: Scene,
{
    /// Inherited scenes.
    pub inherit: I,
    /// Patch that will be constructed and inserted on this entity.
    pub patch: P,
    /// Child scenes of this entity.
    pub children: C,
    /// Optional key used for retaining.
    pub key: Option<Key>,
}

impl<I, P, C> Scene for EntityPatch<I, P, C>
where
    I: Scene,
    P: Patch + DynamicPatch,
    C: Scene,
{
    const ROOT_COUNT: usize = 1;

    /// Constructs an [`EntityPatch`], inserts the resulting bundle to the context entity, and recursively spawns children.
    fn construct(mut self, context: &mut ConstructContext) -> Result<(), ConstructError> {
        if !I::ROOT_COUNT > 0 {
            // Dynamic scene
            let mut dynamic_scene = DynamicScene::default();
            self.dynamic_patch(&mut dynamic_scene);
            dynamic_scene.construct(context)?;
        } else {
            // Static scene
            let bundle = context.construct_from_patch(&mut self.patch)?;
            context.world.entity_mut(context.id).insert(bundle);
            self.children.spawn(context)?;
        }

        Ok(())
    }

    fn spawn(self, context: &mut ConstructContext) -> Result<(), ConstructError> {
        let id = context.world.spawn_empty().id();
        context.world.entity_mut(context.id).add_child(id);

        self.construct(&mut ConstructContext {
            id,
            world: context.world,
        })?;

        Ok(())
    }

    fn dynamic_patch(&mut self, scene: &mut DynamicScene) {
        // Apply the inherited patches
        self.inherit.dynamic_patch(scene);

        // Apply this patch itself
        self.patch.dynamic_patch(scene);

        // Push the children
        self.children.dynamic_patch_as_child(scene);
    }

    /// Dynamically patches the scene and pushes it as a child of the [`DynamicScene`].
    fn dynamic_patch_as_child(&mut self, parent_scene: &mut DynamicScene) {
        let mut child_scene = DynamicScene::default();
        self.dynamic_patch(&mut child_scene);
        parent_scene.push_child(child_scene);
    }
}

/// Extension trait implementing [`Scene`] utilities for [`ConstructContext`].
pub trait ConstructContextSceneExt {
    /// Constructs a [`Scene`], inserts the components to the context entity, and recursively spawns the descendants.
    fn construct_scene(&mut self, scene: impl Scene) -> Result<&mut Self, ConstructError>;

    /// Spawns a [`Scene`] under the context entity recursively.
    fn spawn_scene(&mut self, scene: impl Scene) -> Result<&mut Self, ConstructError>;
}

impl ConstructContextSceneExt for ConstructContext<'_> {
    fn construct_scene(&mut self, scene: impl Scene) -> Result<&mut Self, ConstructError> {
        scene.construct(self)?;
        Ok(self)
    }

    fn spawn_scene(&mut self, scene: impl Scene) -> Result<&mut Self, ConstructError> {
        scene.spawn(self)?;
        Ok(self)
    }
}

struct ConstructSceneCommand<S>(S)
where
    S: Scene + Send + 'static;

impl<S> EntityCommand for ConstructSceneCommand<S>
where
    S: Scene + Send + 'static,
{
    fn apply(self, entity: EntityWorldMut) {
        let mut context = ConstructContext::new(entity.id(), entity.into_world_mut());
        self.0
            .construct(&mut context)
            .expect("failed to spawn_scene in ConstructSceneCommand");
    }
}

/// Extension trait implementing [`Scene`] utilities for [`EntityCommands`].
pub trait EntityCommandsSceneExt {
    /// Constructs a [`Scene`] and applies it to the entity.
    fn construct_scene(&mut self, scene: impl Scene + Send + 'static) -> EntityCommands;
}

impl EntityCommandsSceneExt for EntityCommands<'_> {
    // type Out = EntityCommands;
    fn construct_scene(&mut self, scene: impl Scene + Send + 'static) -> EntityCommands {
        self.queue(ConstructSceneCommand(scene));
        self.reborrow()
    }
}

/// Scene spawning extension.
pub trait SpawnSceneExt {
    /// Spawn the given [`Scene`].
    fn spawn_scene(&mut self, scene: impl Scene + Send + 'static) -> EntityCommands;
}

impl SpawnSceneExt for Commands<'_, '_> {
    /// Spawn the given [`Scene`].
    fn spawn_scene(&mut self, scene: impl Scene + Send + 'static) -> EntityCommands {
        let mut entity = self.spawn_empty();
        entity.queue(ConstructSceneCommand(scene));
        entity
    }
}

impl SpawnSceneExt for ChildSpawnerCommands<'_> {
    fn spawn_scene(&mut self, scene: impl Scene + Send + 'static) -> EntityCommands {
        let mut entity = self.spawn_empty();
        entity.queue(ConstructSceneCommand(scene));
        entity
    }
}
