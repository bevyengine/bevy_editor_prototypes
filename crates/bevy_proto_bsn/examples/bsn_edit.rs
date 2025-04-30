//! Load, edit, save a BSN file
//!
//! In a real world editor, you would want to use a more sophisticated way to edit the BSN tree
//! (likely through an abstraction like [`BsnReflector`] or similar), but this example shows the basic idea.
use bevy::prelude::*;
use bevy_proto_bsn::*;

#[derive(Resource, Default)]
struct EditorState {
    bsn: Handle<Bsn>,
}

#[derive(Component, Default, Reflect)]
struct Counter(i32);

const ASSET: &str = "counter.proto_bsn";
const SAVE_PATH: &str = "assets/counter.proto_bsn";

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BsnPlugin)
        .register_type::<Counter>()
        .init_resource::<EditorState>()
        .add_systems(
            Startup,
            |mut commands: Commands,
             asset_server: Res<AssetServer>,
             mut state: ResMut<EditorState>| {
                state.bsn = asset_server.load(ASSET);

                commands.spawn(Camera2d);
                commands.spawn_empty().construct_scene(pbsn! {
                    Node {
                        position_type: PositionType::Absolute,
                        flex_direction: FlexDirection::Column,
                        padding: px_all(30.0),
                        row_gap: px(10.0),
                    } [
                        Text(format!("Editing '{}'", SAVE_PATH)),
                        Text("R to reload"),
                        Text("S to save"),
                        Text("SPACE to increment counter"),
                    ]
                });
            },
        )
        .add_systems(Update, handle_edit_operations)
        .run();
}

fn handle_edit_operations(
    input: Res<ButtonInput<KeyCode>>,
    state: ResMut<EditorState>,
    asset_server: Res<AssetServer>,
    mut assets: ResMut<Assets<Bsn>>,
) {
    if input.just_pressed(KeyCode::KeyR) {
        asset_server.reload(ASSET);
    }

    let Some(bsn) = assets.get_mut(state.bsn.id()) else {
        return;
    };

    if input.just_pressed(KeyCode::KeyS) {
        std::fs::write(SAVE_PATH, bsn.to_bsn_string()).unwrap();
        info!("Saved to {}", SAVE_PATH);
    }
    if input.just_pressed(KeyCode::Space) {
        let (prev_i, prev_counter) = bsn
            .root
            .components
            .iter()
            .enumerate()
            .find_map(|(i, c)| match c {
                BsnComponent::Patch(path, props) if path == "Counter" => match props {
                    BsnProps::TupleLike(props) => match props.first() {
                        Some(BsnProp::Value(BsnValue::Number(value))) => {
                            Some((Some(i), Counter(value.parse::<i32>().unwrap())))
                        }
                        _ => None,
                    },
                    _ => Some((Some(i), Counter::default())),
                },
                _ => None,
            })
            .unwrap_or_default();

        if let Some(i) = prev_i {
            bsn.root.components.remove(i);
        }

        let new_value = prev_counter.0 + 1;
        bsn.root.components.push(BsnComponent::Patch(
            "Counter".to_string(),
            BsnProps::TupleLike(vec![BsnProp::Value(BsnValue::Number(
                new_value.to_string(),
            ))]),
        ));

        info!("Incremented from {} to {}", prev_counter.0, new_value);
    }
}
