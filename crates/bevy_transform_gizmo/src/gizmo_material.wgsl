#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}

struct GizmoMaterial {
    color: vec4<f32>,
};

@group(2) @binding(0)
var<uniform> material: GizmoMaterial;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = mesh_position_local_to_clip(
        get_world_from_local(vertex.instance_index),
        vec4<f32>(vertex.position, 1.0),
    );
    return out;
}

@fragment
fn fragment() -> @location(0) vec4<f32> {
    return material.color;
}
