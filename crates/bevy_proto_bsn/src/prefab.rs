use std::hash::BuildHasher;

use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    platform_support::{collections::HashMap, hash::FixedState},
    prelude::*,
};

use crate::*;

pub(crate) fn prefab_plugin(app: &mut App) {
    app.register_type::<Prefab>()
        .register_type::<PrefabProps>()
        .register_type::<PrefabInstance>()
        .add_systems(SpawnScene, prefab_system);
}

/// BSN prefab component. Insert this component to spawn a BSN asset.
///
/// If `bevy/file_watcher` is enabled, the instance will be intelligently updated on asset hot reload.
#[derive(Debug, Component, Reflect, Construct)]
#[reflect(Component, Construct)]
#[require(PrefabInstance)]
#[component(immutable, on_insert = on_insert_prefab, on_remove = on_remove_prefab)]
pub struct Prefab(#[construct] pub ConstructHandle<Bsn>);

/// Prefab instance component. Keeps track of the currently retained BSN hash.
#[derive(Debug, Component, Default, Reflect)]
pub struct PrefabInstance {
    current_hash: Option<u64>,
}

fn on_insert_prefab(mut world: DeferredWorld, context: HookContext) {
    world
        .commands()
        .entity(context.entity)
        .queue(|mut entity: EntityWorldMut| {
            entity.get_mut::<PrefabInstance>().unwrap().current_hash = None;
        });
}

fn on_remove_prefab(mut world: DeferredWorld, context: HookContext) {
    world
        .commands()
        .entity(context.entity)
        .remove::<PrefabInstance>()
        .retain_scene_with::<Prefab>(());
}

fn prefab_system(
    mut query: Query<(Entity, &Prefab, &mut PrefabInstance)>,
    mut events: EventReader<AssetEvent<Bsn>>,
    bsn_assets: Res<Assets<Bsn>>,
    app_registry: Res<AppTypeRegistry>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut loaded_hashes: Local<HashMap<AssetId<Bsn>, u64>>,
) {
    // Detect loaded/unloaded BSN assets
    for event in events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                let bsn = bsn_assets.get(*id).unwrap();
                let hash = FixedState::default().hash_one(bsn);
                loaded_hashes.insert(*id, hash);
            }
            AssetEvent::Removed { id } => {
                loaded_hashes.remove(id);
            }
            _ => {}
        }
    }

    // Spawn/retain prefabs
    let registry = app_registry.read();
    for (entity, prefab, mut instance) in query.iter_mut() {
        let asset_id = prefab.0.id();
        let Some(loaded_hash) = loaded_hashes.get(&asset_id) else {
            continue;
        };

        if instance
            .current_hash
            .is_none_or(|current_hash| current_hash != *loaded_hash)
        {
            instance.current_hash = Some(*loaded_hash);

            // TODO: Use a pre-reflected BSN asset instead of reflecting on every spawned instance
            let bsn = bsn_assets.get(asset_id).unwrap();
            let dynamic_scene = BsnReflector::new(bsn, &registry)
                .with_asset_load(&asset_server)
                .reflect_dynamic_scene();
            let dynamic_scene = match dynamic_scene {
                Ok(scene) => scene,
                Err(err) => {
                    error!(
                        "Failed to reflect BSN for {}: {:?}",
                        asset_server
                            .get_path(asset_id)
                            .map(|p| p.to_string())
                            .unwrap_or_else(|| "<unknown path>".to_string()),
                        err
                    );
                    continue;
                }
            };

            commands
                .entity(entity)
                .retain_scene_with::<Prefab>(dynamic_scene);
        }
    }
}
