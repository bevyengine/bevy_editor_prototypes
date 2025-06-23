//! Crate for a Bevy Editor Component Viewer pane. This has yet to be blessed, but I thought it might be nice to specifically view components
//!
//! This crate provides a plugin and UI for viewing and interacting with components in a Bevy application.
use bevy::{
    color::palettes::tailwind, input::common_conditions::input_just_pressed, prelude::*,
    tasks::IoTaskPool,
};
use bevy_pane_layout::prelude::{PaneAppExt, PaneStructure};
use bevy_remote::{
    builtin_methods::{BrpQuery, BrpQueryFilter, BrpQueryParams, BRP_QUERY_METHOD},
    http::{DEFAULT_ADDR, DEFAULT_PORT},
    BrpRequest,
};
use serde_json;
/// Plugin for the editor properties pane.
pub struct ComponentViewerPlugin;

impl Plugin for ComponentViewerPlugin {
    fn build(&self, app: &mut App) {
        app.register_pane("Components", setup_pane).add_systems(
            Update,
            connect_to_remote.run_if(input_just_pressed(KeyCode::Space)),
        );
        // .add_systems(PostUpdate, update_remote_component_viewer_pane);
    }
}

/// Root UI node of the properties pane.
#[derive(Component)]
struct ComponentViewerRoot;

fn setup_pane(pane: In<PaneStructure>, mut commands: Commands) {
    commands.entity(pane.content).insert((
        ComponentViewerRoot,
        Node {
            flex_direction: FlexDirection::Column,
            flex_grow: 1.0,
            column_gap: Val::Px(4.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..Default::default()
        },
        BackgroundColor(tailwind::NEUTRAL_600.into()),
    ));
}

/// Connects to a remote Bevy application via HTTP REST and sends a request to query components.
fn connect_to_remote(_commands: Commands) {
    let task_pool = IoTaskPool::get();
    task_pool
        .spawn(async move {
            let components = vec!["bevy_transform::components::transform::Transform".to_string()];
            let url = format!("http://{}:{}/", DEFAULT_ADDR, DEFAULT_PORT);
            info!("Connecting to remote at {}", url);
            let req = BrpRequest {
                jsonrpc: String::from("2.0"),
                method: String::from(BRP_QUERY_METHOD),
                id: Some(serde_json::to_value(1).expect("Failed to serialize id")),
                params: Some(
                    serde_json::to_value(BrpQueryParams {
                        data: BrpQuery {
                            components: components,
                            option: Vec::default(),
                            has: Vec::default(),
                        },
                        strict: false,
                        filter: BrpQueryFilter::default(),
                    })
                    .expect("Unable to convert query parameters to a valid JSON value"),
                ),
            };

            // For some reason it fails here, with no return or error/response code?
            info!("Sending request: {:#?}", req);
            let res = ureq::post(&url).send_json(req);

            match res {
                Ok(mut response) => match response.body_mut().read_json::<serde_json::Value>() {
                    Ok(json) => info!("Received response: {:#?}", json),
                    Err(e) => error!("Failed to parse JSON response: {}", e),
                },
                Err(e) => error!("Failed to send request: {}", e),
            }
        })
        .detach();
}

// fn update_remote_component_viewer_pane(
//     panes: Query<Entity, With<ComponentViewerRoot>>,
//     world_state: Res<RemoteWorldState>,
//     mut commands: Commands,
// ) {
//     for pane in &panes {
//         commands
//             .entity(pane)
//             .build_children(remote_component_viewer_update(&*world_state));
//     }
// }

// fn remote_component_viewer_update(world_state: &RemoteWorldState) {
//     let mut entity_nodes = Vec::new();
//     for entity in &world_state.entities {
//         let mut component_nodes = Vec::new();
//         for component in &entity.components {
//             component_nodes.push(template! {
//                 Node {
//                     flex_direction: FlextDirection::Row,
//                     ..Default::default()
//                 } => [
//                     (Text(component.type_name.clone()), TextFont::from_font_size(12.0)),
//                     (Text(format!("{:?}", component.data)), TextFont::from_font_size(10.0)),
//                 ];
//             });
//         }
//         entity_nodes.push(template! {
//             Node {
//                 flex_direction, FlexDirection::Column,
//                 margin: UiRect::vertical(Val::Px(4.0)),
//                 ..Default::default()
//             } => [
//                 (Text(format!("Entity {:?}", entity.id)), TextFont::from_font_size(14.0)),
//                 @{ component_nodes };
//             ];
//         });
//     }
// }
