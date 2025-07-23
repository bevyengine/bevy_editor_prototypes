use bevy::{
    prelude::*,
    render::{mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
};
/// A cone shape.
#[derive(Debug, Clone, Copy)]
pub struct Cone {
    pub radius: f32,
    pub height: f32,
    pub subdivisions: usize,
}

impl Default for Cone {
    fn default() -> Self {
        Cone {
            radius: 1.0,
            height: 1.0,
            subdivisions: 32,
        }
    }
}

impl From<Cone> for Mesh {
    fn from(cone: Cone) -> Self {
        // code adapted from http://apparat-engine.blogspot.com/2013/04/procedural-meshes-torus.html
        // (source code at https://github.com/SEilers/Apparat)

        let n_vertices = cone.subdivisions + 2;
        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(n_vertices);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(n_vertices);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(n_vertices);

        let side_stride = 2.0 * std::f32::consts::PI / cone.subdivisions as f32;

        // Cone tip
        positions.push([0.0, cone.height, 0.0]);
        normals.push(Vec3::Y.into());
        uvs.push([0.0, 1.0]);
        // Bottom center
        positions.push([0.0, 0.0, 0.0]);
        normals.push(Vec3::new(0.0, -1.0, 0.0).into());
        uvs.push([0.0, -1.0]);

        for side in 0..=cone.subdivisions {
            let phi = side_stride * side as f32;
            let x = phi.cos() * cone.radius;
            let y = 0.0;
            let z = phi.sin() * cone.radius;

            let vertex = Vec3::new(x, y, z);
            let tangent = vertex.normalize().cross(Vec3::Y).normalize();
            let edge = (Vec3::Y - vertex).normalize();
            let normal = edge.cross(tangent).normalize();

            positions.push([x, y, z]);
            normals.push(normal.into());
            uvs.push([side as f32 / cone.subdivisions as f32, 0.0]);
        }

        let n_triangles = cone.subdivisions * 2;
        let n_indices = n_triangles * 3;

        let mut indices: Vec<u32> = Vec::with_capacity(n_indices);

        for point in 2..cone.subdivisions + 2 {
            let top = 0;
            let bottom = 1;

            let left = point + 1;
            let right = point;

            indices.push(top as u32);
            indices.push(left as u32);
            indices.push(right as u32);

            indices.push(bottom as u32);
            indices.push(right as u32);
            indices.push(left as u32);
        }

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_indices(Indices::U32(indices));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}
