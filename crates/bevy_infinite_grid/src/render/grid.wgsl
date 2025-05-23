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
    projection: mat4x4<f32>,
    inverse_projection: mat4x4<f32>,
    /// Expected to be World from view
    view: mat4x4<f32>,
    /// Expected to be View from world
    inverse_view: mat4x4<f32>,
    world_position: vec3<f32>,
};

@group(0) @binding(0) var<uniform> view: View;

@group(1) @binding(0) var<uniform> grid_position: InfiniteGridPosition;
@group(1) @binding(1) var<uniform> grid_settings: InfiniteGridSettings;

struct Vertex {
    @builtin(vertex_index) index: u32,
};

fn strip_translation(m: mat4x4<f32>) -> mat4x4<f32> {
    return mat4x4<f32>(
        m[0],
        m[1],
        m[2],
        vec4<f32>(0.0, 0.0, 0.0, 1.0)
    );
}

// Transforms point `p` from NDC to World Space Coordinates 
fn unproject_point(p: vec3<f32>) -> vec3<f32> {
    let unprojected = view.view * view.inverse_projection * vec4<f32>(p, 1.0);
    return unprojected.xyz / unprojected.w;
}

// Transforms point `p` from NDC to World Space Coordinates 
// but relative to View's translation
fn unproject_point_no_translation(p: vec3<f32>) -> vec3<f32> {
    let unprojected = strip_translation(view.view) * view.inverse_projection * vec4<f32>(p, 1.0);
    return unprojected.xyz / unprojected.w;
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) ray_origin: vec3<f32>,
    @location(1) ray_direction_near: vec3<f32>,
    @location(2) ray_direction_far: vec3<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    // 0 1 2 1 2 3
    var grid_plane = array<vec3<f32>, 4>(
        vec3<f32>(-1., -1., 1.),
        vec3<f32>(-1., 1., 1.),
        vec3<f32>(1., -1., 1.),
        vec3<f32>(1., 1., 1.)
    );
    let p = grid_plane[vertex.index].xyz;

    var out: VertexOutput;

    out.clip_position = vec4<f32>(p, 1.);
    out.ray_origin = unproject_point(p);
    out.ray_direction_near = unproject_point_no_translation(p);
    out.ray_direction_far = unproject_point_no_translation(vec3<f32>(p.xy, 0.001));
    return out;
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
};

@fragment
fn fragment(in: VertexOutput) -> FragmentOutput {
    let ray_origin = in.ray_origin;
    let ray_direction = normalize(in.ray_direction_far - in.ray_direction_near);
    let plane_normal = grid_position.normal;
    let plane_origin = grid_position.origin;

    let denominator = dot(ray_direction, plane_normal);
    let point_to_point = plane_origin - ray_origin;
    let t = dot(plane_normal, point_to_point) / denominator;
    let frag_pos_3d = ray_direction * t + ray_origin;

    let planar_offset = frag_pos_3d - plane_origin;
    let rotation_matrix = grid_position.planar_rotation_matrix;
    let plane_coords = (grid_position.planar_rotation_matrix * planar_offset).xz;


    let view_space_pos = view.inverse_view * vec4(frag_pos_3d, 1.);
    let clip_space_pos = view.projection * view_space_pos;
    let clip_depth = clip_space_pos.z / clip_space_pos.w;
    let real_depth = -view_space_pos.z;

    var out: FragmentOutput;

    out.depth = clip_depth;

    // Perspective scaling

    let camera_distance_from_plane = abs(dot(view.world_position - plane_origin, plane_normal));

    // The base 10 log of the camera distance
    let log10_distance = log(max(grid_settings.scale, camera_distance_from_plane)) / log(10.);

    // The scaling to be used when the camera projection has perspective
    let perspective_scaling = pow(10., floor(log10_distance));


    // Orthographic scaling

    // The height of the view in world units
    let view_area_height = 2. / view.projection[1][1];

    // Who knows what it means?
    let cool_magic_number = 300.;
    let size = view_area_height / cool_magic_number;

    // The base 10 log of the viewport size
    let log10_size = log(max(1., size)) / log(10.);

    // The scaling to be used when the camera projection is orthographic
    let orthographic_scaling = pow(10., floor(log10_size)) ;


    // Equal to 1 when the camera projection is orthographic. Otherwise 0
    let is_orthographic = view.projection[3].w;

    // Choose different scaling methods for perspective and orthographic projections
    let scaling = mix(perspective_scaling, orthographic_scaling, is_orthographic);

    let scale = grid_settings.scale * scaling;
    let coord = plane_coords / scale; // use the scale variable to set the distance between the lines
    let derivative = fwidth(coord);
    let grid = abs(fract(coord - 0.5) - 0.5) / derivative;
    let lne = min(grid.x, grid.y);

    let minimumz = min(derivative.y, 1.) * scale;
    let minimumx = min(derivative.x, 1.) * scale;

    let derivative2 = fwidth(coord * 0.1);
    let grid2 = abs(fract((coord * 0.1) - 0.5) - 0.5) / derivative2;
    let mg_line = min(grid2.x, grid2.y);

    let grid_alpha = 1.0 - min(lne, 1.0);
    let base_grid_color = mix(grid_settings.major_line_col, grid_settings.minor_line_col, step(1., mg_line));
    var grid_color = vec4(base_grid_color.rgb, base_grid_color.a * grid_alpha);

    let main_axes_half_width = 0.8;
    let z_axis_cond = plane_coords.x > -main_axes_half_width * minimumx && plane_coords.x < main_axes_half_width * minimumx;
    let x_axis_cond = plane_coords.y > -main_axes_half_width * minimumz && plane_coords.y < main_axes_half_width * minimumz;

    grid_color = mix(grid_color, vec4(grid_settings.z_axis_col, grid_color.a), f32(z_axis_cond));
    grid_color = mix(grid_color, vec4(grid_settings.x_axis_col, grid_color.a), f32(x_axis_cond));

    let dist_fadeout = min(1., 1. - grid_settings.dist_fadeout_const / max(1., camera_distance_from_plane / 10.) * real_depth);
    let dot_fadeout = abs(dot(grid_position.normal, normalize(view.world_position - frag_pos_3d)));
    let alpha_fadeout = mix(dist_fadeout, 1., dot_fadeout) * min(grid_settings.dot_fadeout_const * dot_fadeout, 1.);

    grid_color.a = grid_color.a * alpha_fadeout;
    out.color = grid_color;

    return out;
}