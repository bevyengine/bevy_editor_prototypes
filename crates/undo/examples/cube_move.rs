use bevy::prelude::*;
use undo::*;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UndoPlugin)
        .auto_reflected_undo::<Transform>()
        .add_systems(Startup, setup)
        .add_systems(Update, (move_cube, send_undo_event, write_undo_text))
        .run();
}

#[derive(Component)]
struct Controller;

fn setup(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    cmd.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 2.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    })
    .insert(Controller)
    .insert(UndoMarker) //Only entities with this marker will be able to undo
    .insert(OneFrameUndoIgnore::default()); // To prevert add "Transform add" change in change chain

    cmd.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    cmd.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Start,
            ..default()
        },
        ..default()
    }).with_children(|parent| {
        parent.spawn(TextBundle {
            text: Text {
                sections: vec![],
                ..default()
            },
            ..default()
        });
    });
}

fn move_cube(
    inputs : Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Controller>>,
    time: Res<Time>,
) {
    let speed = 10.0;
    if inputs.pressed(KeyCode::A) {
        for mut transform in &mut query {
            transform.translation += Vec3::new(-1.0, 0.0, 0.0) * time.delta_seconds() * speed;
        }
    }

    if inputs.pressed(KeyCode::D) {
        for mut transform in &mut query {
            transform.translation += Vec3::new(1.0, 0.0, 0.0) * time.delta_seconds() * speed;
        }
    }
}

fn send_undo_event(
    mut events: EventWriter<UndoRedo>,
    inputs : Res<Input<KeyCode>>,
) {
    if inputs.just_pressed(KeyCode::Z) && inputs.pressed(KeyCode::ControlLeft) && !inputs.pressed(KeyCode::ShiftLeft) {
        events.send(UndoRedo::Undo);
    }

    if inputs.just_pressed(KeyCode::Z) && inputs.pressed(KeyCode::ControlLeft) && inputs.pressed(KeyCode::ShiftLeft) {
        events.send(UndoRedo::Redo);
    }
}
    
fn write_undo_text(
    mut query: Query<&mut Text>,
    change_chain: Res<ChangeChain>, //Change chain in UndoPlugin
) {
    for mut text in &mut query {
        text.sections.clear();
        text.sections.push(TextSection::new("Registered changes\n", TextStyle::default()));
        for change in change_chain.changes.iter() {
            text.sections.push(TextSection::new(format!("{}\n", change.debug_text()), TextStyle::default()));
        }
    }
}