#import bevy_sprite::mesh2d_vertex_output::VertexOutput;

@group(0) @binding(0) var<uniform> zoom: f32;
@group(1) @binding(1) var<uniform> color: vec4<f32>;
@group(1) @binding(2) var<uniform> grid_cell_size: vec2<f32>;

fn circle_pattern(uv: vec2<f32>, cell_size: vec2<f32>, circle_radius: f32) -> f32 {
    let cell_center = fract(uv / cell_size) - 0.5;
    let dist_from_center = length(cell_center * 2.0); // Scale for proper aspect ratio in cell space
    let in_circle = smoothstep(circle_radius, circle_radius - 0.005, dist_from_center);
    return in_circle;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv;
    let circle_radius = 0.075; // Radius of the circles in the grid pattern
    let circle_value = circle_pattern(uv, grid_cell_size, circle_radius);
    let base_color = vec4<f32>(0.0, 0.0, 0.0, 0.0); // Dark background color
    let final_color = mix(base_color, color, circle_value);

    return final_color;
}