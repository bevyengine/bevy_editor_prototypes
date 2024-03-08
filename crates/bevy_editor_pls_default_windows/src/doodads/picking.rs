



use bevy::{input::keyboard::KeyboardInput, prelude::*};
use bevy_editor_pls_core::Editor;
use crate::hierarchy::HierarchyWindow;
use bevy_inspector_egui::bevy_egui::EguiContexts;
use bevy_mod_raycast::{immediate::Raycast, CursorRay};

use super::{doodad::DoodadComponent, DoodadToolState, PlaceDoodadEvent};

#[derive(Event)]
pub struct SelectDoodadEvent {
	pub entity:Entity
}

#[derive(Component)]
pub struct PreventEditorSelection {}









 pub fn update_picking_doodads(
    mouse_input: Res<ButtonInput<MouseButton>>, //detect mouse click

      key_input: Res<ButtonInput<KeyCode>>,

    cursor_ray: Res<CursorRay>,
    mut raycast: Raycast,

    mut  event_writer: EventWriter<SelectDoodadEvent>,

   mut editor: ResMut<Editor>,  

   unpickable_query: Query<&PreventEditorSelection >,
   doodad_comp_query: Query<&DoodadComponent> ,
   parent_query: Query<&Parent> ,

     
) {
 
   let state = editor.window_state_mut::<HierarchyWindow>().unwrap();
 
 
    
    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }


    //shift mutes doodad selection so you can use the gizmo  
   /* if key_input.pressed(KeyCode::ShiftLeft) {
        return ; 
    }*/

    //must deselect w right click first 
     if !state.selected.is_empty() {
        return ; 
    }
 
   
 
    if let Some(cursor_ray) = **cursor_ray {
        if let Some(( intersection_entity, intersection_data)) =
            raycast.cast_ray(cursor_ray, &default()).first()
        {
            let hit_point = intersection_data.position(); 


            

                if unpickable_query.get( *intersection_entity ).ok().is_some(){
                    println!("This entity is marked as non-selectable");
                    return 
                }
           


             	let mut top_doodad_comp_parent_entity:Option<Entity>  = None; 
 				for parent_entity in AncestorIter::new(&parent_query, *intersection_entity) {

                         if unpickable_query.get( parent_entity ).ok().is_some(){
                            println!("This entity is marked as non-selectable");
                            return 
                         }

 						if  doodad_comp_query.get(parent_entity).ok().is_some() {

 							top_doodad_comp_parent_entity = Some(parent_entity) ;
 							break;

 						} 
 				}
                 println!("select doodad   {:?}", hit_point); 


 				let focus_entity = top_doodad_comp_parent_entity.unwrap_or( intersection_entity.clone()  );

 				 state.selected.select_replace(  focus_entity.clone() )  ; 
            

                 event_writer.send(SelectDoodadEvent { 
                   entity: focus_entity.clone()
                 });
          //  }
           //
        }
    }
}
