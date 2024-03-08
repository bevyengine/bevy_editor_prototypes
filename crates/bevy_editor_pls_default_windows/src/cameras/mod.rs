 
use crate::scenes::NotInScene;

use bevy::render::camera::RenderTarget;
use bevy::render::view::RenderLayers;
use bevy::utils::HashSet;
use bevy::window::WindowRef;
use bevy::{prelude::*, render::primitives::Aabb};
use bevy_editor_pls_core::{
    editor_window::{EditorWindow, EditorWindowContext},
    Editor, EditorEvent,
};
use bevy_inspector_egui::egui;
// use bevy_mod_picking::prelude::PickRaycastSource;

use crate::hierarchy::{HideInEditor, HierarchyWindow};

//use self::camera_3d_panorbit::PanOrbitCamera;

pub const EDITOR_RENDER_LAYER: u8 = 19;

// Present on all editor cameras
#[derive(Component)]
pub struct EditorCamera;

// Present only one the one currently active camera
#[derive(Component)]
pub struct ActiveEditorCamera;

// Marker component for the 3d free camera
#[derive(Component)]
struct EditorCamera3dFree;

// Marker component for the 3d pan+orbit
#[derive(Component)]
struct EditorCamera3dPanOrbit;

// Marker component for the 2d pan+zoom camera
#[derive(Component)]
struct EditorCamera2dPanZoom;

pub struct CameraWindow;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum EditorCamKind {
   // D2PanZoom,
    D3Free,
    #[default]
    D3PanOrbit,
}

impl EditorCamKind {
    fn name(self) -> &'static str {
        match self {
         //   EditorCamKind::D2PanZoom => "2D (Pan/Zoom)",
            EditorCamKind::D3Free => "3D (Free)",
            EditorCamKind::D3PanOrbit => "3D (Pan/Orbit)",
        }
    }

    fn all() -> [EditorCamKind; 2] {
        [
           // EditorCamKind::D2PanZoom,
            EditorCamKind::D3Free,
            EditorCamKind::D3PanOrbit,
        ]
    }
}

#[derive(Default)]
pub struct CameraWindowState {
    // make sure to keep the `ActiveEditorCamera` marker component in sync with this field
    editor_cam: EditorCamKind,
    pub show_ui: bool,
}

impl CameraWindowState {
    pub fn editor_cam(&self) -> EditorCamKind {
        self.editor_cam
    }
}

impl EditorWindow for CameraWindow {
    type State = CameraWindowState;

    const NAME: &'static str = "Cameras";

    fn ui(world: &mut World, _cx: EditorWindowContext, ui: &mut egui::Ui) {
        cameras_ui(ui, world);
    }

    fn viewport_toolbar_ui(world: &mut World, mut cx: EditorWindowContext, ui: &mut egui::Ui) {
       let state = cx.state_mut::<CameraWindow>().unwrap();
      
        ui.checkbox(&mut state.show_ui, "UI");
    }

    fn app_setup(app: &mut App) {
        app.init_resource::<PreviouslyActiveCameras>();

        app
             
           .add_systems(PreUpdate, configure_camera_custom)
         //   .add_systems(PreUpdate, focus_selected)
              //.add_systems(Update, initial_camera_setup)
              ;
       // app.add_systems(PreStartup, spawn_editor_cameras);

        app.add_systems(
            PostUpdate,
            set_main_pass_viewport
                .after(bevy_editor_pls_core::EditorSet::UI)
                .before(bevy::render::camera::CameraUpdateSystem),
        );
    }
} 

fn cameras_ui(ui: &mut egui::Ui, world: &mut World) {
    // let cameras = active_cameras.all_sorted();
    // let mut query: QueryState<&Camera> = world.query();
    // for camera in query.iter(world) {
    //
    // }

    let prev_cams = world.resource::<PreviouslyActiveCameras>();

    ui.label("Cameras");
    for cam in prev_cams.0.iter() {
        ui.horizontal(|ui| {
            // let active = curr_active.or(prev_active);

            /*let text = egui::RichText::new("👁").heading();
            let show_hide_button = egui::Button::new(text).frame(false);
            if ui.add(show_hide_button).clicked() {
                toggle_cam_visibility = Some((camera.to_string(), active));
            }*/

            // if active.is_none() {
            //     ui.set_enabled(false);
            // }

            ui.label(format!("{}: {:?}", "Camera", cam));
        });
    }
}
 
 

fn configure_camera_custom(
    mut commands: Commands,

    mut cam_query: Query<(Entity, &mut Camera), Without<ActiveEditorCamera>>,

  editor: Res<Editor>

    ){

    let Some((cam_entity , mut camera_config)) = cam_query.get_single_mut().ok() else {return};

    let render_layers = RenderLayers::default().with(EDITOR_RENDER_LAYER);

    let target = RenderTarget::Window(WindowRef::Entity(editor.window()));

    camera_config.target = target.clone();

 


    commands.entity(cam_entity)
    .insert( ActiveEditorCamera {} )
    .insert( NotInScene {} )
     .insert( HideInEditor {} )
       .insert( EditorCamera {} )
         .insert( EditorCamera3dFree {} )
         .insert( render_layers )
        //    .insert( Ec3d )



    ;

}


#[derive(Resource, Default)]
struct PreviouslyActiveCameras(HashSet<Entity>);

fn toggle_editor_cam(
    editor: Res<Editor>,
    mut editor_events: EventReader<EditorEvent>,
    mut prev_active_cams: ResMut<PreviouslyActiveCameras>,
    mut cam_query: Query<(Entity, &mut Camera)>,
) {
    if editor.always_active() {
        //Prevent accumulation of irrelevant events
        editor_events.clear();
        return;
    }

    for event in editor_events.read() {
        if let EditorEvent::Toggle { now_active } = *event {
            if now_active {
                // Add all currently active cameras
                for (e, mut cam) in cam_query
                    .iter_mut()
                    //  Ignore non-Window render targets
                    .filter(|(_e, c)| matches!(c.target, RenderTarget::Window(_)))
                    .filter(|(_e, c)| c.is_active)
                {
                    prev_active_cams.0.insert(e);
                    cam.is_active = false;
                }
            } else {
                for cam in prev_active_cams.0.iter() {
                    if let Ok((_e, mut camera)) = cam_query.get_mut(*cam) {
                        camera.is_active = true;
                    }
                }
                prev_active_cams.0.clear();
            }
        }
    }
}

  

fn set_main_pass_viewport(
    egui_settings: Res<bevy_inspector_egui::bevy_egui::EguiSettings>,
    editor: Res<Editor>,
    window: Query<&Window>,
    mut cameras: Query<&mut Camera, With<EditorCamera>>,
) {
    if !editor.is_changed() {
        return;
    };

    let Ok(window) = window.get(editor.window()) else {
        return;
    };

    let viewport = editor.active().then(|| {
        let scale_factor = window.scale_factor() * egui_settings.scale_factor;

        let viewport_pos = editor.viewport().left_top().to_vec2() * scale_factor;
        let viewport_size = editor.viewport().size() * scale_factor;

        if !viewport_size.is_finite() {
            warn!("editor viewport size is infinite");
        }

        bevy::render::camera::Viewport {
            physical_position: UVec2::new(viewport_pos.x as u32, viewport_pos.y as u32),
            physical_size: UVec2::new(
                (viewport_size.x as u32).max(1),
                (viewport_size.y as u32).max(1),
            ),
            depth: 0.0..1.0,
        }
    });

    cameras.iter_mut().for_each(|mut cam| {
        cam.viewport = viewport.clone();
    });
}
