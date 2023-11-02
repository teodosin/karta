struct InfiniteGrid {
    origin: vec3<f32>,
    scale: f32,
    color: Vec3,
};




@group(0) @binding(0)
var<uniform> infinite_grid: InfiniteGrid;



struct Vertex {
    @builtin(vertex_index) index: u32,
};

fn unproject_point(p: vec3<f32>) -> vec3<f32> {
    let unprojected = view.view * view.inverse_projection * vec4<f32>(p, 1.0);
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
    var grid_plane = array<vec3<f32>, 4>(
        vec3<f32>(-1., -1., 1.),
        vec3<f32>(-1., 1., 1.),
        vec3<f32>(1., -1., 1.),
        vec3<f32>(1., 1., 1.)
    );
    let p = grid_plane[vertex.index].xyz;

    var out: VertexOutput;

    out.clip_position = vec4<f32>(p, 1.);
    out.near_point = unproject_point(p);
    out.far_point = unproject_point(vec3<f32>(p.xy, 0.001)); // unprojecting on the far plane
    return out;
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
};

@fragment
fn fragment(in: VertexOutput) -> FragmentOutput {
    let plane_origin = infinite_grid.origin;

    var out: FragmentOutput;

    out.depth = clip_depth;

    let scale = infinite_grid.scale;
    let coord = plane_coords * scale; // use the scale variable to set the distance between the lines
    let derivative = fwidth(coord);
    let grid = abs(fract(coord - 0.5) - 0.5) / derivative;
    let lne = min(grid.x, grid.y);

    let minimumz = min(derivative.y, 1.) / scale;
    let minimumx = min(derivative.x, 1.) / scale;

    let derivative2 = fwidth(coord * 0.1);
    let grid2 = abs(fract((coord * 0.1) - 0.5) - 0.5) / derivative2;
    let mg_line = min(grid2.x, grid2.y);

    let grid_alpha = 1.0 - min(lne, 1.0);
    let base_grid_color = mix(infinite_grid.major_line_col, infinite_grid.minor_line_col, step(1., mg_line));
    let grid_color = vec4<f32>(base_grid_color.rgb, base_grid_color.a * grid_alpha);


    let z_axis_cond = plane_coords.x > -1.0 * minimumx && plane_coords.x < 1.0 * minimumx;
    let x_axis_cond = plane_coords.y > -1.0 * minimumz && plane_coords.y < 1.0 * minimumz;

    color = infinite_grid.color;

    color.a = color.a * alpha_fadeout;
    out.color = color;

    return out;
}