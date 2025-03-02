use bevy::{
    ecs::{component::HookContext, system::error_handler, world::DeferredWorld},
    prelude::*,
};

use crate::*;

pub(crate) fn prefab_plugin(app: &mut App) {
    app.register_type::<Prefab>()
        .register_type::<PrefabProps>()
        .register_type::<PrefabInstance>()
        .add_systems(SpawnScene, prefab_system);
}

/// BSN prefab component. Insert this component to spawn a BSN asset instance.
///
/// If `bevy/file_watcher` is enabled, the instance will be intelligently updated on asset hot reload.
#[derive(Debug, Component, Reflect, Construct)]
#[reflect(Component, Construct)]
#[require(PrefabInstance)]
#[component(immutable, on_insert = on_insert_prefab, on_remove = on_remove_prefab)]
pub struct Prefab(#[construct] pub ConstructHandle<ReflectedBsn>);

/// Prefab instance component. Keeps track of the currently retained BSN hash.
#[derive(Debug, Component, Default, Reflect)]
pub struct PrefabInstance {
    current_hash: Option<u64>,
}

fn on_insert_prefab(mut world: DeferredWorld, context: HookContext) {
    world.commands().entity(context.entity).queue_handled(
        |mut entity: EntityWorldMut| {
            entity.get_mut::<PrefabInstance>().unwrap().current_hash = None;
        },
        error_handler::silent(),
    );
}

fn on_remove_prefab(mut world: DeferredWorld, context: HookContext) {
    world.commands().entity(context.entity).queue_handled(
        |mut entity: EntityWorldMut| {
            entity
                .remove::<PrefabInstance>()
                .retain_scene_with::<Prefab>(())
                .unwrap();
        },
        error_handler::silent(),
    );
}

fn prefab_system(
    mut query: Query<(Entity, &Prefab, &mut PrefabInstance)>,
    reflected_bsn_assets: Res<Assets<ReflectedBsn>>,
    mut commands: Commands,
) {
    // Spawn/retain prefabs
    for (entity, prefab, mut instance) in query.iter_mut() {
        let asset_id = prefab.0.id();
        let Some(bsn) = reflected_bsn_assets.get(asset_id) else {
            return;
        };

        if instance
            .current_hash
            .is_none_or(|current_hash| current_hash != bsn.hash)
        {
            instance.current_hash = Some(bsn.hash);

            let bsn = reflected_bsn_assets.get(asset_id).unwrap();
            let scene = bsn.clone().into_dynamic_scene();

            commands.entity(entity).retain_scene_with::<Prefab>(scene);
        }
    }
}
