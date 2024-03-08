## Bevy Mesh Terrain Editor 



![image](https://github.com/ethereumdegen/bevy_mesh_terrain_editor/assets/6249263/9e32f8a0-e513-4ee0-8b4b-3e4d73ab8608)



Load, Edit, and Save terrain files for bevy_mesh_terrain in a standalone application 



## How to use 

 


### Tips and tricks 

- You dont have to 'save all chunks' unless you need to export collision data to a game.  Often, saving splat and height is sufficient and far faster. 

- When painting, the system supports up to 255 textures. However, you have to  be very careful how you blend them.  To blend, be sure that you use the 'layer fade' and fade between two textures at every transition or you will get artifact lines.  This technique does make painting slightly more tedius but offers extreme splat map optimization and texture capacity in your game. 






### Placing Doodads 

 - Create a file at assets/doodad_manifest.manifest.ron  and build your doodad definitions in there 

```
 # this is an example doodad manifest file telling the editor how to render (preview) doodads 
 # see doodad_manifest.rs and zone_file.rs for more information about how this works 
  (
    doodad_definitions: [
        (
            name: "birch_yellow_1",
            model: GltfModel("models/doodads/birch_yellow.glb"),
        ) ,
        (
            name: "bonfire",
            model: GltfModel("models/doodads/bonfire.glb"),            ,
            initial_custom_props: Some({ "my_prop": Float(1.0) })
        ) 
    ]
  )


```


### Zones 

- Using the Zones window, you can spawn a zone entity.  Right click on this zone entity in the hierarchy to set it as the primary zone.  When a zone is primary, placed doodads will become children of the zone


- Right click on the zone entity in the hierarchy to save the zone to a file.  You can use the zone window to load zone files back in later.  

