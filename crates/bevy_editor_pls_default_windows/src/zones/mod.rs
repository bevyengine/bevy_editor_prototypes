use bevy::prelude::*;
use bevy_editor_pls_core::editor_window::{EditorWindow, EditorWindowContext};
use bevy_inspector_egui::egui::{self, RichText};

use std::path::Path;

use crate::doodads::PlaceDoodadEvent;

#[derive(Component)]
pub struct ZoneComponent {

}


#[derive(Event)]
pub enum ZoneEvent {
	SetZoneAsPrimary(Entity),
	SaveZoneToFile(Entity),
	CreateNewZone(String),
	LoadZoneFile(String),
	ResetPrimaryZone
}

#[derive(Resource,Default)]
pub struct ZoneResource {
	pub primary_zone: Option<Entity>,

}

pub mod zone_file;

use zone_file::ZoneFile;

use self::zone_file::CustomPropsComponent;



const DEFAULT_FILENAME: &str = "zone01";

#[derive(Default, Component)]
pub struct NotInScene;

#[derive(Default)]
pub struct ZoneWindowState {
    create_filename: String,
    load_filename:String,
    zone_create_result: Option< Result<(), Box<dyn std::error::Error + Send + Sync>>>,
}

pub struct ZoneWindow;

impl EditorWindow for ZoneWindow {
    type State = ZoneWindowState;
    const NAME: &'static str = "Zones";

    fn ui(world: &mut World, mut cx: EditorWindowContext, ui: &mut egui::Ui) {
        let state = cx.state_mut::<ZoneWindow>().unwrap();

        let zone_resource = world.resource::<ZoneResource>();
        let primary_zone_entity = zone_resource.primary_zone;

         let primary_zone_name = primary_zone_entity
        .and_then(|ent| {
            // Temporarily fetch the component to avoid holding the borrow
            world.get::<Name>(ent).map(|n| n.as_str().to_owned())
        })
        .unwrap_or_else(|| "None".to_owned());

      

		 ui.vertical(|ui| {

		 	 ui.horizontal(|ui| {
				 ui.label( format!("Primary zone: {:?}", primary_zone_name.clone()   )) ;
				  if ui.button("Reset").clicked()   {
				  		world.send_event::<ZoneEvent>( ZoneEvent::ResetPrimaryZone ) ;
				  }
			});



		 //create zone 
	        ui.horizontal(|ui| { 


	               let res = egui::TextEdit::singleline(&mut state.create_filename)
	                .hint_text(DEFAULT_FILENAME)
	                .desired_width(120.0)
	                .show(ui);



	            if res.response.changed() {
	                state.zone_create_result = None;
	            }

	            let enter_pressed = ui.input(|input| input.key_pressed(egui::Key::Enter));

	            if ui.button("Create Zone").clicked() || enter_pressed {
	                let create_filename = if state.create_filename.is_empty() {
	                    DEFAULT_FILENAME
	                } else {
	                    &state.create_filename
	                };
	                let mut query = world.query_filtered::<Entity, Without<NotInScene>>();
	               // let entitys = query.iter(world).collect();
	                state.zone_create_result = Some(create_zone(world , create_filename));
	            }

	        });



	           ui.horizontal(|ui| { 


	               let res = egui::TextEdit::singleline(&mut state.load_filename)
	                .hint_text(DEFAULT_FILENAME)
	                .desired_width(120.0)
	                .show(ui);



	            if res.response.changed() {
	                state.zone_create_result = None;
	            }

	            let enter_pressed = ui.input(|input| input.key_pressed(egui::Key::Enter));

	            if ui.button("Load Zone").clicked() || enter_pressed {
	                let load_filename = if state.load_filename.is_empty() {
	                    DEFAULT_FILENAME
	                } else {
	                    &state.load_filename
	                };
	                let mut query = world.query_filtered::<Entity, Without<NotInScene>>();
	               // let entitys = query.iter(world).collect();
	                state.zone_create_result = Some(load_zone(world, load_filename));
	            }

	        })
	        // ----- h
	    }); // ---- v

        if let Some(status) = &state.zone_create_result {
            match status {
                Ok(()) => {
                    ui.label(RichText::new("Success!").color(egui::Color32::GREEN));
                }
                Err(error) => {
                    ui.label(RichText::new(error.to_string()).color(egui::Color32::RED));
                }
            }
        }
    

     }
}




fn create_zone(
  //  world: &mut World,
   world: &mut World,
    name: &str,
   
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>{

	 world.send_event::<ZoneEvent>(
     	ZoneEvent::CreateNewZone(name.into()));
    
   Ok(())
}


fn load_zone(
    world: &mut World,
    name: &str,
   
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
     
     world.send_event::<ZoneEvent>(
     	ZoneEvent::LoadZoneFile(name.into()));
  
    Ok(())
}


pub fn handle_zone_events( 

	mut commands: Commands,
  mut evt_reader: EventReader<ZoneEvent>,

  mut zone_resource: ResMut<ZoneResource>,

  children_query: Query<&Children, With<Name> >,

  zone_entity_query: Query<( &Name, &Transform, Option<&CustomPropsComponent>)>,

  mut spawn_doodad_event_writer: EventWriter<PlaceDoodadEvent>

 ){


 	for evt in evt_reader.read(){



  match evt {
    ZoneEvent::CreateNewZone(name) => {

		   let created_zone = commands.spawn(SpatialBundle::default())
		   .insert(ZoneComponent{})
		   .insert(Name::new( name.to_string() ))
		   .id();

  		  zone_resource.primary_zone = Some( created_zone );
    }

    ZoneEvent::SetZoneAsPrimary(ent) =>  {

    	zone_resource.primary_zone = Some(ent.clone());

    },
    ZoneEvent::ResetPrimaryZone => {
    	zone_resource.primary_zone = None; 
    },
    ZoneEvent::SaveZoneToFile(ent) => {	


    		//this is kind of wacky but we are using this as a poor mans name query 
    	let Some((zone_name_comp, _ , _ )) = zone_entity_query.get(ent.clone()).ok() else {return};

    	let zone_name :&str = zone_name_comp.as_str();

    	let mut all_children :Vec<Entity> = Vec::new();

    	for child in DescendantIter::new(&children_query, ent.clone()) {
    		all_children.push(child);
    	}

    	  

    	let zone_file = ZoneFile::new(all_children,&zone_entity_query);

    	let zone_file_name = format!("{}.zone.ron", zone_name);


    	 let ron = ron::ser::to_string(&zone_file).unwrap();
  		 let file_saved = std::fs::write(zone_file_name, ron);


  		  println!("exported zone ! {:?}", file_saved);



    },

     ZoneEvent::LoadZoneFile(zone_name) => {	

     	let file_name = format!("{}.zone.ron", zone_name);

     	 let path = Path::new(&file_name);

	    // Read the file into a string
	    let Ok(file_content) = std::fs::read_to_string(path) else {
	    	println!("Could not find file {:?}",file_name);
	    	return
	    };

	    // Deserialize the string into ZoneFile
	    let Ok(zone_file)  = ron::from_str::<ZoneFile>(&file_content) else {
	    	println!("Could not parse file {:?}",file_name);
	    	return
	    };


	    //spawnn the zone entity and set it as primary 

	    let created_zone = commands.spawn(SpatialBundle::default())
			   .insert(ZoneComponent{})
			   .insert(Name::new( zone_name.to_string() ))
			   .id();

		zone_resource.primary_zone = Some( created_zone );

	    //trigger spawn doodad events 

	    for zone_entity in zone_file.entities {

	    	spawn_doodad_event_writer.send({

	    		PlaceDoodadEvent{
	    			doodad_name: zone_entity.name .clone(),
	    			position: zone_entity.get_position(),
	    			rotation_euler: Some(zone_entity.get_rotation_euler()),
	    			scale: Some(zone_entity.get_scale())

	    		}

	    	});

	    }

     }

}



}




}