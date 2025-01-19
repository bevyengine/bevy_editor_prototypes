use bevy::prelude::{
    BuildChildren, ChildBuild, ChildBuilder, Commands, Entity, EntityCommand, EntityCommands, World,
};
use variadics_please::all_tuples;

use crate::{
    ConstructContext, ConstructContextPatchExt, ConstructError, DynamicPatch, DynamicScene, Patch,
};

/// Destination trait for [`EntityPatch`].
pub trait Scene: Sized {
    /// Constructs a [`Scene`], inserts the components to the context entity, and recursively spawns scene descendants.
    fn construct(self, context: &mut ConstructContext) -> Result<(), ConstructError>;

    /// Constructs and spawns a [`Scene`] as a child under the context entity recursively.
    fn spawn(self, context: &mut ConstructContext) -> Result<(), ConstructError>;

    /// Dynamically applies the patches of this scene to a [`DynamicScene`], effectively overwriting any patched props.
    fn dynamic_patch(&mut self, scene: &mut DynamicScene);

    /// Dynamically patches the scene and pushes it as a child of the [`DynamicScene`].
    fn dynamic_patch_as_child(&mut self, scene: &mut DynamicScene);
}

/// Zero or more [`Scene`]es forming a set of children or inherited patches. Implemented for tuples of [`Scene`].
pub trait SceneTuple {
    /// Whether this is an empty tuple
    const IS_EMPTY: bool;

    /// Recursively constructs/spawns all the entities in the tuple and their descendants under the context entity.
    fn spawn_children(self, context: &mut ConstructContext) -> Result<(), ConstructError>;

    /// Applies each scene in the tuple to the dynamic scene by calling [`Scene::dynamic_patch`].
    fn dynamic_patch(&mut self, scene: &mut DynamicScene);

    /// Pushes the scenes in the tuple as children of the dynamic scene.
    fn push_dynamic_children(&mut self, scene: &mut DynamicScene);
}

impl SceneTuple for () {
    const IS_EMPTY: bool = true;

    fn spawn_children(self, _: &mut ConstructContext) -> Result<(), ConstructError> {
        Ok(())
    }

    fn dynamic_patch(&mut self, _: &mut DynamicScene) {}

    fn push_dynamic_children(&mut self, _: &mut DynamicScene) {}
}

// Tuple impls
macro_rules! impl_scene_tuple {
    ($(#[$meta:meta])* $(($S:ident, $s:ident)),*) => {
        $(#[$meta])*
        impl<$($S: Scene),*> SceneTuple for ($($S,)*)
        {
            const IS_EMPTY: bool = false;

            fn spawn_children(self, context: &mut ConstructContext) -> Result<(), ConstructError> {
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

            fn push_dynamic_children(&mut self, scene: &mut DynamicScene) {
                let ($($s,)*) = self;
                $($s.dynamic_patch_as_child(scene);)*
            }
        }
    };
}

all_tuples!(
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
    I: SceneTuple,
    P: Patch + DynamicPatch,
    C: SceneTuple,
{
    /// Inherited scenes.
    pub inherit: I,
    /// Patch that will be constructed and inserted on this entity.
    pub patch: P,
    /// Child scenes of this entity.
    pub children: C,
}

impl<I, P, C> Scene for EntityPatch<I, P, C>
where
    I: SceneTuple,
    P: Patch + DynamicPatch,
    C: SceneTuple,
{
    /// Constructs an [`EntityPatch`], inserts the resulting bundle to the context entity, and recursively spawns children.
    fn construct(mut self, context: &mut ConstructContext) -> Result<(), ConstructError> {
        if !I::IS_EMPTY {
            // Dynamic scene
            let mut dynamic_scene = DynamicScene::default();
            self.dynamic_patch(&mut dynamic_scene);
            dynamic_scene.construct(context)?;
        } else {
            // Static scene
            let bundle = context.construct_from_patch(&mut self.patch)?;
            context.world.entity_mut(context.id).insert(bundle);
            self.children.spawn_children(context)?;
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
        self.children.push_dynamic_children(scene);
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
    fn apply(self, id: Entity, world: &mut World) {
        let mut context = ConstructContext { id, world };
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

impl SpawnSceneExt for ChildBuilder<'_> {
    fn spawn_scene(&mut self, scene: impl Scene + Send + 'static) -> EntityCommands {
        let mut entity = self.spawn_empty();
        entity.queue(ConstructSceneCommand(scene));
        entity
    }
}
