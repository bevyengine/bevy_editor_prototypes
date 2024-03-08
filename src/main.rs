use bevy::input::mouse::MouseMotion;

use std::f32::consts::PI;


use bevy::prelude::*;
use bevy::render::settings::{RenderCreation, WgpuFeatures, WgpuSettings};
use bevy::render::RenderPlugin;
use bevy_mesh_terrain::edit::EditingTool;
use bevy_mesh_terrain::terrain_config::TerrainConfig;
use bevy_mesh_terrain::{
    edit::{EditTerrainEvent, TerrainCommandEvent},
    terrain::{TerrainData, TerrainViewer},
    TerrainMeshPlugin,
};

use bevy::pbr::ShadowFilteringMethod;

use bevy_mod_raycast::prelude::*;

mod camera;
mod commands;
mod tools;
mod ui;
mod editor_pls;

use crate::camera::camera_plugin;

use crate::tools::brush_tools_plugin;

use crate::commands::update_commands;
use crate::ui::editor_ui_plugin;


use seldom_fn_plugin::FnPluginExt;

 




fn main() {



 let mut wgpu_settings = WgpuSettings::default();
    wgpu_settings.features |= WgpuFeatures::POLYGON_MODE_LINE;

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: bevy::window::PresentMode::AutoNoVsync, //improves latency

                title: "Mesh Terrain Editor".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        })
       

        .set(RenderPlugin {
            render_creation: RenderCreation::Automatic(wgpu_settings),
            ..default()
        }))

        .add_plugins(DefaultRaycastingPlugin)
        .add_plugins(TerrainMeshPlugin::default())
        .fn_plugin(brush_tools_plugin)
        .fn_plugin(editor_ui_plugin)
         .fn_plugin(camera_plugin)
        .add_systems(Startup, setup)
        //move to brushes and tools lib
        
        .add_systems(Update, update_commands)
        //move to camera lib
        .add_plugins(editor_pls::editor_ui_plugin)
       
        .run();
}

/// set up a simple 3D scene
fn setup(mut commands: Commands, // asset_server: Res<AssetServer>
) {
    commands
        .spawn(SpatialBundle::default())
        .insert(
            TerrainConfig::load_from_file("assets/terrain/default_terrain/terrain_config.ron")
                .unwrap(),
        )
        .insert(TerrainData::new());

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadow_depth_bias: 0.5,
            shadow_normal_bias: 0.5,

             illuminance: light_consts::lux::OVERCAST_DAY,
              shadows_enabled: true,

            color: Color::WHITE,
            ..default()
        },

         transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },

        ..default()
    });
    // light
   /* commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,

            shadow_depth_bias: 0.5,
            shadow_normal_bias: 0.5,

            color: Color::WHITE,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 800.0, 4.0),
        ..default()
    });
*/
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: light_consts::lux::OVERCAST_DAY,
    });

    // camera
    
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(20.0, 162.5, 20.0)
                .looking_at(Vec3::new(900.0, 0.0, 900.0), Vec3::Y),
            ..default()
        })
        .insert(TerrainViewer::default())
        .insert(ShadowFilteringMethod::Jimenez14);
         
}
