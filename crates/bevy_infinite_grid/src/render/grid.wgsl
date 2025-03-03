struct InfiniteGridPosition {
    planar_rotation_matrix: mat3x3<f32>,
    origin: vec3<f32>,
    normal: vec3<f32>,

};

struct InfiniteGridSettings {
    scale: f32,
    // 1 / fadeout_distance
    dist_fadeout_const: f32,
    dot_fadeout_const: f32,
    x_axis_col: vec3<f32>,
    z_axis_col: vec3<f32>,
    minor_line_col: vec4<f32>,
    major_line_col: vec4<f32>,

};

struct View {
    clip_from_view: mat4x4<f32>,
    view_from_clip: mat4x4<f32>,
    world_from_view: mat4x4<f32>,
    view_from_world: mat4x4<f32>,
    world_position: vec3<f32>,
    world_right: vec3<f32>,
    world_forward: vec3<f32>,
};

@group(0) @binding(0) var<uniform> view: View;

@group(1) @binding(0) var<uniform> grid_position: InfiniteGridPosition;
@group(1) @binding(1) var<uniform> grid_settings: InfiniteGridSettings;

struct Vertex {
    @builtin(vertex_index) index: u32,
};

fn unproject_point(p: vec3<f32>) -> vec3<f32> {
    let unprojected = view.world_from_view * view.view_from_clip * vec4<f32>(p, 1.0);
    return unprojected.xyz / unprojected.w;
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) near_point: vec3<f32>,
    @location(1) far_point: vec3<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    // 0 1 2 1 2 3
    var grid_plane = array(
        vec3(-1., -1., 1.),
        vec3(-1., 1., 1.),
        vec3(1., -1., 1.),
        vec3(1., 1., 1.),
    );
    let p = grid_plane[vertex.index].xyz;

    var out: VertexOutput;

    out.clip_position = vec4(p, 1.);
    out.near_point = unproject_point(p);
    out.far_point = unproject_point(vec3(p.xy, 0.001)); // unprojecting on the far plane
    return out;
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
};

fn raycast_plane(plane_origin: vec3<f32>, plane_normal: vec3<f32>, ray_origin: vec3<f32>, ray_direction: vec3<f32>) -> vec3<f32> {
    let denominator = dot(ray_direction, plane_normal);
    let point_to_point = plane_origin - ray_origin;
    let t = dot(plane_normal, point_to_point) / denominator;
    return ray_direction * t + ray_origin;
}

fn log10(a: f32) -> f32 {
    return log(a) / log(10.);
}

@fragment
fn fragment(in: VertexOutput) -> FragmentOutput {
    let ray_origin = in.near_point;
    let ray_direction = normalize(in.far_point - in.near_point);
    let plane_normal = grid_position.normal;
    let plane_origin = grid_position.origin;

    let frag_pos_3d = raycast_plane(plane_origin, plane_normal, ray_origin, ray_direction);

    let planar_offset = frag_pos_3d - plane_origin;
    let rotation_matrix = grid_position.planar_rotation_matrix;
    let plane_coords = (grid_position.planar_rotation_matrix * planar_offset).xz;

    // TODO Handle ray misses/NaNs

    // To scale the grid, we need to know how far the camera is from the grid plane. The naive
    // solution is to simply use the distance, however this breaks down when changing FOV or
    // when using an orthographic projection.
    //
    // Instead, we want a solution that is related to the size of objects on screen.

    // Cast a ray from the camera to the plane and get the point where the ray hits the plane.
    let point_a = raycast_plane(plane_origin, plane_normal, view.world_position, view.world_forward);

    // Then we offset that hit one world-space unit in the direction of the camera's right.
    let point_b = point_a + view.world_right;

    // Convert the points to view space
    let view_space_point_a = view.view_from_world * vec4(point_a, 1.);
    let view_space_point_b = view.view_from_world * vec4(point_b, 1.);
    // Take the flat distance between the points in view space
    let view_space_distance = distance(view_space_point_a.xy, view_space_point_b.xy);

    // Finally, we use the relationship that the scale of an object is inversely proportional to
    // the distance from the camera. We can now do the reverse - compute a distance based on the
    // size in the view. If we are very far from the plane, the two points will be very close
    // in the view, if we are very close to the plane, the two objects will be very far apart
    // in the view. This will work for any camera projection regardless of the camera's
    // translational distance.
    let log10_scale = log10(max(grid_settings.scale, 1. / view_space_distance));

    // Floor the scaling to the nearest power of 10.
    let scaling = pow(10., floor(log10_scale));

    let view_space_pos = view.view_from_world * vec4(frag_pos_3d, 1.);
    let clip_space_pos = view.clip_from_view * view_space_pos;
    let clip_depth = clip_space_pos.z / clip_space_pos.w;
    let real_depth = -view_space_pos.z;

    var out: FragmentOutput;

    out.depth = clip_depth;

    let camera_distance_from_plane = abs(dot(view.world_position - plane_origin, plane_normal));

    let scale = grid_settings.scale * scaling;
    let coord = plane_coords / scale; // use the scale variable to set the distance between the lines
    let derivative = fwidth(coord);
    let grid = abs(fract(coord - 0.5) - 0.5) / derivative;
    let lne = min(grid.x, grid.y);

    let minimumz = min(derivative.y, 1.) * scale;
    let minimumx = min(derivative.x, 1.) * scale;

    let derivative2 = fwidth(coord * 0.1);
    let grid2 = abs(fract(coord * 0.1 - 0.5) - 0.5) / derivative2;
    let is_minor_line = step(1., min(grid2.x, grid2.y));
    let minor_alpha_multiplier = 1. - fract(log10_scale);

    let grid_alpha = 1.0 - min(lne, 1.0);
    let base_grid_color = mix(grid_settings.major_line_col, grid_settings.minor_line_col * vec4(1., 1., 1., minor_alpha_multiplier), is_minor_line);
    var grid_color = vec4(base_grid_color.rgb, base_grid_color.a * grid_alpha);

    let main_axes_half_width = 0.8;
    let z_axis_cond = plane_coords.x > -main_axes_half_width * minimumx && plane_coords.x < main_axes_half_width * minimumx;
    let x_axis_cond = plane_coords.y > -main_axes_half_width * minimumz && plane_coords.y < main_axes_half_width * minimumz;

    grid_color = mix(grid_color, vec4(grid_settings.z_axis_col, grid_color.a), f32(z_axis_cond));
    grid_color = mix(grid_color, vec4(grid_settings.x_axis_col, grid_color.a), f32(x_axis_cond));

    let dist_fadeout = min(1., 1. - grid_settings.dist_fadeout_const / max(1., camera_distance_from_plane / 10.) * real_depth);
    let dot_fadeout = abs(dot(grid_position.normal, normalize(view.world_position - frag_pos_3d)));
    let alpha_fadeout = mix(dist_fadeout, 1., dot_fadeout) * min(grid_settings.dot_fadeout_const * dot_fadeout, 1.);

    grid_color.a *= alpha_fadeout;
    out.color = grid_color;

    return out;
}