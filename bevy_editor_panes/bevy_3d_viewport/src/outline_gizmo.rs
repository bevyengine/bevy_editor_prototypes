use bevy::prelude::*;
use bevy_editor_core::SelectedEntity;

pub struct OutlineGizmoPlugin;
impl Plugin for OutlineGizmoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShowOutlines>()
            .add_systems(Startup, spawn_gizmo_toggle_ui)
            .add_systems(Update, outline_gizmo_system)
            .add_systems(Update, update_gizmo_toggle_text);
    }
}

#[derive(Resource, Default)]
pub struct ShowOutlines(pub bool);

// Marker for the toggle button text
#[derive(Component)]
struct GizmoToggleText;

pub fn outline_gizmo_system(
    show: Res<ShowOutlines>,
    query: Query<&Transform>,
    selected_entity: Res<SelectedEntity>,
    mut gizmos: Gizmos,
) {
    if !show.0 {
        return;
    }
    if let Some(entity) = selected_entity.0 {
        if let Ok(transform) = query.get(entity) {
            gizmos.cuboid(*transform, Color::srgb(1.0, 0.0, 0.0));
        }
    }
}

pub fn spawn_gizmo_toggle_ui(mut commands: Commands) {
    info!("Spawning Gizmo Toggle UI");
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                right: Val::Px(20.0),
                width: Val::Px(100.0),
                height: Val::Px(15.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Show Outlines"),
                TextFont::from_font_size(10.0),
                GizmoToggleText,
            ));
        })
        .observe(
            |_trigger: Trigger<Pointer<Click>>, mut show_outlines: ResMut<ShowOutlines>| {
                show_outlines.0 = !show_outlines.0;
            },
        );
}

// System to update the button text when ShowOutlines changes
fn update_gizmo_toggle_text(
    show_outlines: Res<ShowOutlines>,
    mut query: Query<&mut Text, With<GizmoToggleText>>,
) {
    if show_outlines.is_changed() {
        for mut text in &mut query {
            text.0 = if show_outlines.0 {
                "Hide Outlines".into()
            } else {
                "Show Outlines".into()
            };
        }
    }
}
