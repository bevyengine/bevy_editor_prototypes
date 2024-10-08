//! Contains example for spawning many numeric fields

use bevy::{math::VectorSpace, prelude::*};
use bevy_focus::Focus;
use bevy_numeric_field::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultNumericFieldPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, set_values)
        .run();
}

#[derive(Resource, Clone)]
struct TransformWidget {
    transform: [Entity; 3],
    scale: [Entity; 3],
    rotation: [Entity; 4],
    target: Entity,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(Camera3dBundle {
        camera: Camera {
            // color from bevy website background
            clear_color: ClearColorConfig::Custom(Color::srgb(
                34.0 / 255.0,
                34.0 / 255.0,
                34.0 / 255.0,
            )),
            ..default()
        },
        transform: Transform::from_translation(Vec3::splat(5.0)).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 5.0, 1.0))
            .looking_at(Vec3::ZERO, Vec3::X),
        ..default()
    });

    let target = commands
        .spawn(PbrBundle {
            mesh: meshes.add(Cuboid::from_length(1.0)),
            material: materials.add(StandardMaterial::default()),
            ..default()
        })
        .id();

    let translate_x = spawn_field(&mut commands, |val, transform| {
        transform.translation.x = val;
    });
    let translate_y = spawn_field(&mut commands, |val, transform| {
        transform.translation.y = val;
    });
    let translate_z = spawn_field(&mut commands, |val, transform| {
        transform.translation.z = val;
    });

    let scale_x = spawn_field(&mut commands, |val, transform| {
        transform.scale.x = val;
    });
    let scale_y = spawn_field(&mut commands, |val, transform| {
        transform.scale.y = val;
    });
    let scale_z = spawn_field(&mut commands, |val, transform| {
        transform.scale.z = val;
    });

    let rotation_x = spawn_field(&mut commands, |val, transform| {
        transform.rotation.x = val;
    });
    let rotation_y = spawn_field(&mut commands, |val, transform| {
        transform.rotation.y = val;
    });
    let rotation_z = spawn_field(&mut commands, |val, transform| {
        transform.rotation.z = val;
    });
    let rotation_w = spawn_field(&mut commands, |val, transform| {
        transform.rotation.w = val;
    });

    let transform_widget = TransformWidget {
        transform: [translate_x, translate_y, translate_z],
        scale: [scale_x, scale_y, scale_z],
        rotation: [rotation_x, rotation_y, rotation_z, rotation_w],
        target,
    };

    let root = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(25.0),
                height: Val::Percent(100.0),
                display: Display::Grid,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                grid_template_columns: vec![GridTrack::min_content(), GridTrack::flex(1.0)],
                grid_template_rows: vec![GridTrack::min_content(); 13],
                ..Default::default()
            },
            ..Default::default()
        })
        .id();

    header_text(&mut commands, root, "Translate");

    child_text(&mut commands, root, "X");
    commands.entity(root).add_child(translate_x);
    child_text(&mut commands, root, "Y");
    commands.entity(root).add_child(translate_y);
    child_text(&mut commands, root, "Z");
    commands.entity(root).add_child(translate_z);

    header_text(&mut commands, root, "Scale");

    child_text(&mut commands, root, "X");
    commands.entity(root).add_child(scale_x);
    child_text(&mut commands, root, "Y");
    commands.entity(root).add_child(scale_y);
    child_text(&mut commands, root, "Z");
    commands.entity(root).add_child(scale_z);

    header_text(&mut commands, root, "Rotation");

    child_text(&mut commands, root, "X");
    commands.entity(root).add_child(rotation_x);
    child_text(&mut commands, root, "Y");
    commands.entity(root).add_child(rotation_y);
    child_text(&mut commands, root, "Z");
    commands.entity(root).add_child(rotation_z);
    child_text(&mut commands, root, "W");
    commands.entity(root).add_child(rotation_w);

    commands.insert_resource(transform_widget);
}

fn spawn_field(
    commands: &mut Commands,
    callback: impl Fn(f32, &mut Transform) + Send + Sync + 'static,
) -> Entity {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Px(300.0),
                    height: Val::Px(30.0),
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                ..Default::default()
            },
            NumericField::<f32>::new(0.0),
        ))
        .observe(
            move |trigger: Trigger<NewValue<f32>>,
                  mut q_transforms: Query<&mut Transform>,
                  widget: Res<TransformWidget>| {
                let Ok(mut transform) = q_transforms.get_mut(widget.target) else {
                    return;
                };
                callback(trigger.event().0, &mut transform);
            },
        )
        .id()
}

fn child_text(commands: &mut Commands, root: Entity, text: &str) {
    let id = commands
        .spawn(TextBundle::from_section(text, TextStyle::default()))
        .id();
    commands.entity(root).add_child(id);
}

fn header_text(commands: &mut Commands, root: Entity, text: &str) {
    let id = commands
        .spawn(
            TextBundle::from_section(text, TextStyle::default()).with_style(Style {
                grid_column: GridPlacement::span(2),
                ..default()
            }),
        )
        .id();
    commands.entity(root).add_child(id);
}

fn set_values(
    mut commands: Commands,
    widget: Res<TransformWidget>,
    q_transforms: Query<&Transform, Changed<Transform>>,
) {
    let Ok(transform) = q_transforms.get(widget.target) else {
        return;
    };

    commands.trigger_targets(SetValue(transform.translation.x), widget.transform[0]);
    commands.trigger_targets(SetValue(transform.translation.y), widget.transform[1]);
    commands.trigger_targets(SetValue(transform.translation.z), widget.transform[2]);
    commands.trigger_targets(SetValue(transform.scale.x), widget.scale[0]);
    commands.trigger_targets(SetValue(transform.scale.y), widget.scale[1]);
    commands.trigger_targets(SetValue(transform.scale.z), widget.scale[2]);
    commands.trigger_targets(SetValue(transform.rotation.x), widget.rotation[0]);
    commands.trigger_targets(SetValue(transform.rotation.y), widget.rotation[1]);
    commands.trigger_targets(SetValue(transform.rotation.z), widget.rotation[2]);
    commands.trigger_targets(SetValue(transform.rotation.w), widget.rotation[3]);
}
