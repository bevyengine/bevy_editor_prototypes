use bevy::{
    asset::ReflectAsset,
   
    reflect::TypeRegistry,
};


use bevy::prelude::*;
use rand::Rng;


use bevy::{
     gltf::{Gltf, GltfMesh, GltfNode} };


use bevy_editor_pls_core::editor_window::{EditorWindow, EditorWindowContext};
use bevy_editor_pls_core::Editor;
use crate::doodads::doodad_manifest::RenderableType;
use crate::placement::PlacementWindow;
use crate::zones::zone_file::CustomPropsComponent;
use crate::zones::ZoneResource;
use bevy_inspector_egui::bevy_egui::EguiContexts;
use bevy_inspector_egui::{   egui::{self, ScrollArea}};

 
use bevy_common_assets::ron::RonAssetPlugin;

use bevy_mod_raycast::CursorRay;

use bevy_mod_raycast::prelude::Raycast;

use self::doodad::{DoodadComponent, DoodadPlugin, LoadedGltfAssets};
use self::doodad_manifest::{DoodadDefinition, DoodadManifest, DoodadManifestResource};



pub mod picking;
pub mod doodad;
mod doodad_manifest;



#[derive(Resource,Default)]
pub struct DoodadToolState  {
   pub selected: Option<DoodadDefinition> ,
   
}  
  



#[derive(Event)]
 pub struct PlaceDoodadEvent {

    pub position: Vec3,
    pub scale: Option<Vec3>,
    pub rotation_euler: Option<Vec3> ,
    pub doodad_name: String ,

   // pub doodad_definition: DoodadDefinition
 }
 




#[derive(Default)]
pub struct DoodadWindowState  {
  //  pub selected: Option<DoodadDefinition> ,
  //  rename_info: Option<RenameInfo>,
}


pub struct DoodadsWindow;

impl EditorWindow for DoodadsWindow {
    type State = DoodadWindowState;
    const NAME: &'static str = "Doodads";

     /// Necessary setup (resources, systems) for the window.
    fn app_setup(app: &mut App) {
       app
        
            .add_plugins(RonAssetPlugin::<DoodadManifest>::new(&["manifest.ron"]))

              .add_plugins( DoodadPlugin  )
          .insert_resource( DoodadManifestResource::default()  ) 
          .insert_resource( DoodadToolState::default()  ) 
            .insert_resource( LoadedGltfAssets::default()  ) 
          .add_systems(Startup, load_doodad_manifest)
          .add_systems(Update, load_doodad_models)

       
       

       ;
    }



    fn ui(
        world: &mut World,
         mut cx: EditorWindowContext,
          ui: &mut egui::Ui, 

          ) {


        let doodad_definition_resource = world.resource::<DoodadManifestResource>() ;

        //this releases the lock on World 
        let doodad_manifest_handle = &doodad_definition_resource.manifest.clone();


        let doodad_manifests_map = world.resource::<Assets<DoodadManifest>>();

        let doodad_manifest = doodad_manifest_handle.as_ref().and_then(|h|   doodad_manifests_map.get( h ) ) .cloned() ;


        let mut doodad_tool_resource = world.resource_mut::<DoodadToolState>();



         



/*
         let doodad_row_state = match cx.state_mut::<DoodadsWindow >() {
                Some(a) => a,
                None => {
                    let a = cx
                        .state_mut ::<DoodadsWindow   >()
                        .unwrap();
                    a
                }
            };
*/


        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                 


                    let Some(doodad_manifest) = &doodad_manifest else {

                         ui.label(format!(" No doodad definitions found. "));
                         return;

                     };


                     if let Some(selected_doodad_def) = &doodad_tool_resource.selected {
                        ui.label(format!( "Placing: {:?}",  selected_doodad_def.name.clone() ))  ;
                      
                        ui.separator();

                        if ui.button("reset").clicked() {
                             doodad_tool_resource.selected = None;

                        }

                     }else{
                        ui.label( "---"  );
                     }


                      ui.separator();

              

                            for doodad_definition in &doodad_manifest.doodad_definitions {
                                 let label_text = doodad_definition.name.clone();
                                  let checked = false;

                                   if ui.selectable_label(checked, label_text.clone()).clicked() {
                                      //*selection = InspectorSelection::Entities ;

                                   println!("detected a doodad click  !! {:?}", label_text);

                                

                                   doodad_tool_resource.selected = Some( doodad_definition.clone() );

                                }
                            }
                           
                             
            });






    }
}


// --------------------------------------------------------

fn load_doodad_manifest(
   asset_server: Res<AssetServer> , 
   mut doodad_manifest_resource: ResMut<DoodadManifestResource>
){

    doodad_manifest_resource.manifest = Some( asset_server.load("doodad_manifest.manifest.ron")  ) ;
 

}
 
fn load_doodad_models(
     mut evt_asset: EventReader<AssetEvent<DoodadManifest>>,
       doodad_manifest_resource: Res<DoodadManifestResource>,
       doodad_manifest_assets: Res<Assets<DoodadManifest>>,


       mut loaded_gltf_resource: ResMut<LoadedGltfAssets>,

        asset_server: ResMut<AssetServer>
){

    let Some(  doodad_manifest_handle ) = &doodad_manifest_resource.manifest else {return};

    for evt  in evt_asset.read() {

        match evt {
            AssetEvent::LoadedWithDependencies { id } => {

                 if  id == &doodad_manifest_handle.id()  {

                     let manifest: &DoodadManifest = doodad_manifest_assets
                        .get( doodad_manifest_handle.id())
                        .unwrap();

                        println!("loading gltfs 1 ");

                        for doodad_definition in &manifest.doodad_definitions {

                                let model_path = match &doodad_definition.model{
                                    RenderableType::GltfModel(model_path) => model_path
                                };

                                let gltf_model_handle:Handle<Gltf> = asset_server.load( model_path   ) ;

                                loaded_gltf_resource.gltf_models.insert(model_path.clone(), gltf_model_handle); 

                                println!("loaded gltf {:?}", model_path);

                        }

                  
                 }

            }
            _ => {}
        }

     
      

    }
     
 

}
 
  

pub fn handle_place_doodad_events(
    mut commands : Commands,

    mut evt_reader: EventReader<PlaceDoodadEvent>,

    zone_resource: Res<ZoneResource>,

     doodad_manifest_resource: Res<DoodadManifestResource>,
     doodad_manifest_assets: Res<Assets<DoodadManifest>>,

    


) {


     


    let Some(manifest_handle) = &doodad_manifest_resource.manifest else {
        println!("WARN: no doodad manifest file found");
        return
    };

    let Some(manifest) = doodad_manifest_assets.get(manifest_handle) else {
        println!("WARN: no doodad manifest file found");
        return
    };


  

    for evt in evt_reader.read()  {


        let position = &evt.position;
        let doodad_name = &evt.doodad_name;


        let Some(doodad_definition) = manifest.get_doodad_definition_by_name( doodad_name ) else {
            println!("WARN: Could not spawn doodad {:?}",doodad_name);
            continue
        } ;


        let init_custom_props = doodad_definition.initial_custom_props.clone();

        let mut transform = Transform::from_xyz(position.x, position.y, position.z);

        if let Some(rot) = evt.rotation_euler {
           transform = transform.with_rotation( Quat::from_euler(EulerRot::YXZ, rot.x, rot.y, rot.z))
        }
        if let Some(scale) = evt.scale {
           transform = transform.with_scale( scale )
        }


        let doodad_spawned = commands.spawn(SpatialBundle{
            transform ,
            ..default()
        })
        .insert(DoodadComponent::from_definition( &doodad_definition ))
        .id();

        println!("doodad spawned {:?}", doodad_spawned);

        if let Some(init_custom_props) = init_custom_props{
             println!("insert custom props {:?}", init_custom_props);


            commands.entity(doodad_spawned)
            .insert(CustomPropsComponent { props: init_custom_props }) ;
        }
 
        if let Some(primary_zone) = &zone_resource.primary_zone {
            if let Some( mut  ent) = commands.get_entity(primary_zone.clone()) {
                ent.add_child(doodad_spawned);
            }
        }



    }





}


 pub fn update_place_doodads(
    mouse_input: Res<ButtonInput<MouseButton>>, //detect mouse click

    cursor_ray: Res<CursorRay>,
    mut raycast: Raycast,

    mut  event_writer: EventWriter<PlaceDoodadEvent>,
    
    doodad_tool_resource: Res<DoodadToolState>,
 
   // mut contexts: EguiContexts,

        editor: Res<Editor>
) {


  


 

    // ------- compute our rotation and scale from placement properties 
    let placement_window_state = editor.window_state::<PlacementWindow>().unwrap();
    
    let using_random_yaw = placement_window_state.randomize_yaw;
    let random_scale_multiplier = placement_window_state.random_scale_multiplier;

     let mut rng = rand::thread_rng();

    let rotation_euler:Option<Vec3> = match using_random_yaw {

        true =>{ 

            let random_f32 = rng.gen_range(0.0..1.0);
            Some( (random_f32*3.14,0.0,0.0) .into() ) 
        },
        false => None

    };

    let scale :Option<Vec3> = match random_scale_multiplier >= 0.001 {

        true => {


            let random_f32 = rng.gen_range(-1.0..1.0);
            let random_scaled_f32 = 1.0 + random_scale_multiplier * random_f32;

            Some((random_scaled_f32,random_scaled_f32,random_scaled_f32) .into()) 

          },

        false => None
    };
    // -------------------------


    let selected_doodad_definition = &doodad_tool_resource.selected;
    
    let Some(doodad_definition) =  selected_doodad_definition .clone() else {return};
    
    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    } 
  
    if let Some(cursor_ray) = **cursor_ray {
        if let Some((_intersection_entity, intersection_data)) =
            raycast.cast_ray(cursor_ray, &default()).first()
        {
            let hit_point = intersection_data.position();

            

            //offset this by the world psn offset of the entity !? would need to query its transform ?  for now assume 0 offset.
            let hit_coordinates = Vec3::new(hit_point.x, hit_point.y, hit_point.z);

            //use an event to pass the entity and hit coords to the terrain plugin so it can edit stuff there

         

                  //   println!("place doodad 4 {:?}", doodad_definition);

                 event_writer.send(PlaceDoodadEvent { 
                    position: hit_coordinates,
                    doodad_name: doodad_definition.name,
                    rotation_euler,
                    scale 
                 });
            
           
        }
    }
}




 pub fn reset_place_doodads(
    mouse_input: Res<ButtonInput<MouseButton>>, //detect mouse click

      
    
   mut doodad_tool_resource: ResMut<DoodadToolState>,
 
   //  mut contexts: EguiContexts,
) {

   
 
 
   //let egui_ctx = contexts.ctx_mut();
   /*
    if egui_ctx.is_pointer_over_area() {
        return;
    }
 
    */

    if !mouse_input. pressed(MouseButton::Right) {
        return;
    }
 
    doodad_tool_resource.selected = None;
}


