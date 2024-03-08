use bevy::prelude::*;

use bevy::input::mouse::MouseMotion;
use bevy_mesh_terrain::edit::EditingTool;
use bevy_mesh_terrain::terrain_config::TerrainConfig;
use bevy_mesh_terrain::{
    edit::{EditTerrainEvent, TerrainCommandEvent},
    terrain::{TerrainData, TerrainViewer},
    TerrainMeshPlugin,
};

use bevy_mod_raycast::prelude::*;

pub fn update_commands(
    key_input: Res<ButtonInput<KeyCode>>, //detect mouse click

    mut edit_event_writer: EventWriter<EditTerrainEvent>,
    mut command_event_writer: EventWriter<TerrainCommandEvent>,
) {
    if key_input.pressed(KeyCode::ControlLeft) || key_input.pressed(KeyCode::ControlRight) {
        if key_input.just_pressed(KeyCode::KeyS) {
            println!("saving chunks !");

            command_event_writer.send(TerrainCommandEvent::SaveAllChunks(true, true, true));
        }
    }
}
