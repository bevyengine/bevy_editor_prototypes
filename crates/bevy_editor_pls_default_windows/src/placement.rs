use bevy::prelude::*;
use bevy_editor_pls_core::editor_window::{EditorWindow, EditorWindowContext};
use bevy_inspector_egui::egui::{self, RichText};


  
/*
#[derive(Resource,Default)]
pub struct PlacementResource {
	//pub random_scale_multiplier: f32,  //usually like 0.2  .. 0 means no randomness 
	//pub randomize_yaw: bool ,

	//pub translation_grid_lock_step: f32 
}

*/



 
#[derive(Default)]
pub struct PlacementWindowState {
	  pub randomize_yaw: bool ,
	  pub random_scale_multiplier: f32,
	  pub translation_grid_lock_step: f32,
 
}

pub struct PlacementWindow;

impl EditorWindow for PlacementWindow {
    type State = PlacementWindowState;
    const NAME: &'static str = "Placement";

    fn ui(world: &mut World, mut cx: EditorWindowContext, ui: &mut egui::Ui) {
        let state = cx.state_mut::<PlacementWindow>().unwrap();

      //  let placement_resource = world.resource::<PlacementResource>();
      

		 ui.vertical(|ui| {
 


 
	    ui.label("Randomize Rotation (Yaw)");
        if ui.checkbox(&mut state.randomize_yaw, "").changed() {
            // state.randomize_yaw = !state.randomize_yaw;
        }
        ui.end_row();

        ui.label("Random Scale Multiplier");
        let mut scale_mult = state.random_scale_multiplier;
        if ui
            .add(
                egui::DragValue::new(&mut scale_mult)
                    .clamp_range(0..=1)
                    .speed(0.01),
            )
            .changed()
        {
            state.random_scale_multiplier = scale_mult;
        }
        ui.end_row();

          ui.label("Translation Grid Lock Step");
        let mut lock_step = state.translation_grid_lock_step;
        if ui
            .add(
                egui::DragValue::new(&mut lock_step)
                    .clamp_range(0..=10)
                    .speed(0.1),
            )
            .changed()
        {
            state.translation_grid_lock_step = lock_step;
        }


	    }); // ---- v

        
    

     }
}



   

 