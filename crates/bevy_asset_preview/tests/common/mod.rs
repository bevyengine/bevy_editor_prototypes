use bevy::{
    asset::AssetPath,
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};
use bevy_asset_preview::{AssetLoadCompleted, PreviewCache};
use gltf::json::{
    Accessor, Buffer, Mesh, Node, Root, Scene, Value,
    accessor::{ComponentType, GenericComponentType, Type},
    buffer::{Target, View},
    mesh::{Mode, Primitive, Semantic},
    validation::{Checked::Valid, USize64},
};

pub fn create_test_image(
    images: &mut Assets<Image>,
    width: u32,
    height: u32,
    color: [u8; 4],
) -> Handle<Image> {
    let pixel_data: Vec<u8> = (0..(width * height))
        .flat_map(|_| color.iter().copied())
        .collect();

    let image = Image::new_fill(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &pixel_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    images.add(image)
}

pub fn save_test_image(
    image: &Image,
    path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let dynamic_image = image
        .clone()
        .try_into_dynamic()
        .map_err(|e| format!("Failed to convert image: {:?}", e))?;

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();

    match ext.as_str() {
        "jpg" | "jpeg" => {
            let rgb_image = dynamic_image.into_rgb8();
            rgb_image.save(path)?;
        }
        _ => {
            let rgba_image = dynamic_image.into_rgba8();
            rgba_image.save(path)?;
        }
    }
    Ok(())
}

pub fn create_test_model() -> (Root, Vec<u8>) {
    // Simple cube vertices (position only)
    let positions: Vec<[f32; 3]> = vec![
        // Front face
        [-0.5, -0.5, 0.5],
        [0.5, -0.5, 0.5],
        [0.5, 0.5, 0.5],
        [-0.5, 0.5, 0.5],
        // Back face
        [-0.5, -0.5, -0.5],
        [-0.5, 0.5, -0.5],
        [0.5, 0.5, -0.5],
        [0.5, -0.5, -0.5],
        // Top face
        [-0.5, 0.5, -0.5],
        [-0.5, 0.5, 0.5],
        [0.5, 0.5, 0.5],
        [0.5, 0.5, -0.5],
        // Bottom face
        [-0.5, -0.5, -0.5],
        [0.5, -0.5, -0.5],
        [0.5, -0.5, 0.5],
        [-0.5, -0.5, 0.5],
        // Right face
        [0.5, -0.5, -0.5],
        [0.5, 0.5, -0.5],
        [0.5, 0.5, 0.5],
        [0.5, -0.5, 0.5],
        // Left face
        [-0.5, -0.5, -0.5],
        [-0.5, -0.5, 0.5],
        [-0.5, 0.5, 0.5],
        [-0.5, 0.5, -0.5],
    ];

    // Indices for triangles
    let indices: Vec<u16> = vec![
        0, 1, 2, 0, 2, 3, // Front
        4, 5, 6, 4, 6, 7, // Back
        8, 9, 10, 8, 10, 11, // Top
        12, 13, 14, 12, 14, 15, // Bottom
        16, 17, 18, 16, 18, 19, // Right
        20, 21, 22, 20, 22, 23, // Left
    ];

    let mut root = Root::default();

    // Create buffer with positions and indices
    let positions_bytes: &[u8] = bytemuck::cast_slice(&positions);
    let indices_bytes: &[u8] = bytemuck::cast_slice(&indices);
    let mut buffer_data = Vec::from(positions_bytes);
    buffer_data.extend_from_slice(indices_bytes);

    // Pad buffer to multiple of 4
    while buffer_data.len() % 4 != 0 {
        buffer_data.push(0);
    }

    let buffer = root.push(Buffer {
        byte_length: USize64::from(buffer_data.len()),
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        uri: None, // glb doesn't use uri
    });

    // Buffer view for positions
    let positions_view = root.push(View {
        buffer,
        byte_length: USize64::from(positions_bytes.len()),
        byte_offset: Some(USize64(0)),
        byte_stride: None,
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        target: Some(Valid(Target::ArrayBuffer)),
    });

    // Buffer view for indices
    let indices_view = root.push(View {
        buffer,
        byte_length: USize64::from(indices_bytes.len()),
        byte_offset: Some(USize64::from(positions_bytes.len())),
        byte_stride: None,
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        target: Some(Valid(Target::ElementArrayBuffer)),
    });

    // Accessor for positions
    let positions_accessor = root.push(Accessor {
        buffer_view: Some(positions_view),
        byte_offset: None,
        count: USize64::from(positions.len()),
        component_type: Valid(GenericComponentType(ComponentType::F32)),
        extensions: Default::default(),
        extras: Default::default(),
        type_: Valid(Type::Vec3),
        min: Some(Value::from(vec![-0.5, -0.5, -0.5])),
        max: Some(Value::from(vec![0.5, 0.5, 0.5])),
        name: None,
        normalized: false,
        sparse: None,
    });

    // Accessor for indices
    let indices_accessor = root.push(Accessor {
        buffer_view: Some(indices_view),
        byte_offset: None,
        count: USize64::from(indices.len()),
        component_type: Valid(GenericComponentType(ComponentType::U16)),
        extensions: Default::default(),
        extras: Default::default(),
        type_: Valid(Type::Scalar),
        min: None,
        max: None,
        name: None,
        normalized: false,
        sparse: None,
    });

    // Create mesh primitive
    let primitive = Primitive {
        attributes: {
            let mut map = std::collections::BTreeMap::new();
            map.insert(Valid(Semantic::Positions), positions_accessor);
            map
        },
        extensions: Default::default(),
        extras: Default::default(),
        indices: Some(indices_accessor),
        material: None,
        mode: Valid(Mode::Triangles),
        targets: None,
    };

    let mesh = root.push(Mesh {
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        primitives: vec![primitive],
        weights: None,
    });

    let node = root.push(Node {
        mesh: Some(mesh),
        ..Default::default()
    });

    root.scenes = vec![Scene {
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        nodes: vec![node],
    }];

    (root, buffer_data)
}

pub fn save_test_model(
    root: &Root,
    buffer_data: &[u8],
    path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    use gltf::json::serialize;
    use std::borrow::Cow;

    // Serialize to JSON
    let json_string = serialize::to_string(root)?;
    let mut json_offset = json_string.len();
    while json_offset % 4 != 0 {
        json_offset += 1;
    }

    // Create GLB structure
    let glb = gltf::binary::Glb {
        header: gltf::binary::Header {
            magic: *b"glTF",
            version: 2,
            length: (json_offset + buffer_data.len())
                .try_into()
                .map_err(|_| "File size exceeds binary glTF limit")?,
        },
        bin: Some(Cow::Borrowed(buffer_data)),
        json: Cow::Owned({
            let mut json_bytes = json_string.into_bytes();
            while json_bytes.len() % 4 != 0 {
                json_bytes.push(0x20); // pad with space
            }
            json_bytes
        }),
    };

    // Write to file
    let writer = std::fs::File::create(path)?;
    glb.to_writer(writer)?;

    Ok(())
}

pub fn wait_for<F>(app: &mut App, condition: F, max_iterations: usize) -> bool
where
    F: Fn(&App) -> bool,
{
    // Check condition before first update
    if condition(app) {
        return true;
    }

    for _ in 0..max_iterations {
        app.update();
        if condition(app) {
            return true;
        }
    }

    false
}

pub fn wait_for_preview_cached(
    app: &mut App,
    asset_path: &AssetPath<'static>,
    resolution: Option<u32>,
    max_iterations: usize,
) -> bool {
    wait_for(
        app,
        |app| {
            let world = app.world();
            let cache = world.resource::<PreviewCache>();
            cache.get_by_path(asset_path, resolution).is_some()
        },
        max_iterations,
    )
}

pub fn wait_for_load_completion(
    app: &mut App,
    expected_count: usize,
    max_iterations: usize,
) -> Vec<AssetLoadCompleted> {
    let mut loaded_count = 0;
    let mut processed_task_ids = std::collections::HashSet::new();
    let mut completed_events = Vec::new();
    let mut iterations = 0;

    while loaded_count < expected_count && iterations < max_iterations {
        app.update();
        iterations += 1;

        let world = app.world();
        let load_events = world.resource::<Events<AssetLoadCompleted>>();
        let mut cursor = load_events.get_cursor();
        for event in cursor.read(load_events) {
            if processed_task_ids.insert(event.task_id) {
                loaded_count += 1;
                completed_events.push(event.clone());
            }
        }
    }

    assert!(
        loaded_count >= expected_count,
        "Expected at least {} load completions, got {} after {} iterations",
        expected_count,
        loaded_count,
        iterations
    );

    completed_events
}
