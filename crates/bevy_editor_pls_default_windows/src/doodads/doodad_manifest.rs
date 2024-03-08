

use std::{fs::{self, File}, io::{Read, Write}, path::PathBuf};

use bevy::prelude::*;

use serde::{Deserialize, Serialize};

use crate::zones::zone_file::CustomPropsMap;


#[derive(Resource,Default)]
pub struct DoodadManifestResource{
    pub manifest: Option<Handle<DoodadManifest>> 
 
}

 #[derive(Asset , TypePath, Clone, Debug , Serialize, Deserialize )]
pub struct DoodadManifest {

	pub doodad_definitions: Vec<DoodadDefinition>


}


impl DoodadManifest {


    pub fn get_doodad_definition_by_name(&self,name: &str) -> Option<DoodadDefinition> {

        //maybe use a hashmap for this ? 
        for doodad_definition in &self.doodad_definitions {

            if doodad_definition.name == name {
                return Some(doodad_definition.clone())
            }

        }


        None 


    }

}



#[derive(Clone, Debug , Serialize, Deserialize )]
pub enum RenderableType {

    GltfModel(String) //the path 

}

#[derive(Component,Clone, Debug , Serialize, Deserialize )]
pub struct DoodadDefinition{

    pub name: String,
    pub model: RenderableType ,
    pub initial_custom_props: Option<CustomPropsMap>

}

impl DoodadManifest {

  

   pub fn load(  ) -> Result<Self, Box<dyn std::error::Error>> {
        let file_path = get_save_file_path(   ) ;
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let data: Self = ron::de::from_str(&contents)?;
        Ok(data)
    }
}



fn get_save_file_path( ) -> String {

	format!("assets/doodad_manifest.ron"   ) 

}