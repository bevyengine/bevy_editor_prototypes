//! Simple non-destructive .bsn-hot-reloading utilizing i-cant-believe-its-not-bsn `Fragment`s.
//!
//! Run with `--features="bevy/file_watcher"` to enable hot-reloading.
use std::collections::HashSet;

use bevy::{ecs::component::ComponentId, prelude::*, utils::TypeIdMap};
use bevy_bsn::{DynamicScene, *};
use bevy_i_cant_believe_its_not_bsn::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BsnPlugin)
        .add_systems(
            Startup,
            |mut commands: Commands, assets: Res<AssetServer>| {
                commands.spawn(Camera2d);

                let bsn = assets.load::<Bsn>("hello.bsn");
                commands.spawn(SceneRoot(bsn));
            },
        )
        .add_systems(Update, spawn_and_reload_scene)
        .run();
}

#[derive(Component)]
#[allow(dead_code)]
struct SceneRoot(Handle<Bsn>);

fn spawn_and_reload_scene(
    mut events: EventReader<AssetEvent<Bsn>>,
    bsn_assets: Res<Assets<Bsn>>,
    app_registry: Res<AppTypeRegistry>,
    mut commands: Commands,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                let bsn = bsn_assets.get(*id).unwrap();

                let registry = app_registry.read();
                let dynamic_scene = BsnReflector::new(bsn, &registry)
                    .reflect_dynamic_scene()
                    .unwrap();
                let fragment = fragment_from_dynamic_scene(dynamic_scene);

                commands.queue(move |world: &mut World| {
                    let id = world
                        .query_filtered::<Entity, With<SceneRoot>>()
                        .single(world);
                    fragment.build(id, world);
                });
            }
            _ => {}
        }
    }
}

fn fragment_from_dynamic_scene(dynamic_scene: DynamicScene) -> Fragment {
    let (component_props, children) = dynamic_scene.deconstruct();

    Fragment {
        bundle: BoxedBundle::new(DynamicSceneBundle(component_props)),
        children: children
            .into_iter()
            .map(fragment_from_dynamic_scene)
            .collect(),
        ..Default::default()
    }
}

struct DynamicSceneBundle(TypeIdMap<ComponentProps>);

impl ErasedBundle for DynamicSceneBundle {
    fn build(
        self: Box<Self>,
        entity_id: Entity,
        world: &mut World,
        current_components: HashSet<ComponentId>,
    ) -> HashSet<ComponentId> {
        // Collect the new component type ids
        let new_components = self.0.keys().copied().collect::<Vec<_>>();

        // Construct and insert the components
        let mut context = ConstructContext::new(entity_id, world);
        for (_, component_props) in self.0 {
            component_props.construct(&mut context).unwrap();
        }

        // Get the new component ids
        let new_components = new_components
            .into_iter()
            .map(|type_id| world.components().get_id(type_id).unwrap())
            .collect::<HashSet<_>>();

        // Remove the components in the previous bundle but not this one
        let mut entity = world.entity_mut(entity_id);
        for component_id in current_components.difference(&new_components) {
            entity.remove_by_id(*component_id);
        }

        new_components
    }
}
