#import bevy_sprite::mesh2d_vertex_output::VertexOutput;

@group(0) @binding(0) var<uniform> zoom: f32;
@group(1) @binding(1) var<uniform> color: vec4<f32>;
@group(1) @binding(2) var<uniform> grid_cell_size: vec2<f32>;

fn circle_pattern(uv: vec2<f32>, cell_size: vec2<f32>, circle_radius: f32) -> f32 {
    let zoom_index = floor(log2(zoom));

    let cell_uv = uv / cell_size;
    let cell_index = floor(cell_uv);
    let cell_center = fract(cell_uv) - 0.5;
    let dist_from_center = length(cell_center * 4.0); // Scale for proper aspect ratio in cell space

    let actual_radius = circle_radius;

    let in_circle = smoothstep(actual_radius, actual_radius - 0.005, dist_from_center);
    return in_circle;
}

// fn circle_pattern(uv: vec2<f32>, cell_size: vec2<f32>, circle_radius: f32) -> f32 {
//     let zoom_index = floor(zoom);

//     // let cell_sz = cell_size * 5.0 * zoom_index;
//     let cell_sz = cell_size;

//     let cell_uv = uv / cell_sz;
//     let cell_index = floor(cell_uv);
//     let cell_center = fract(cell_uv) - 0.5;
//     let dist_from_center = length(cell_center * 4.0); // Scale for proper aspect ratio in cell space

//     // Make every 5th circle larger
//     let cell_index_fractional = fract(cell_index);
//     let cell_index_integral = cell_index - cell_index_fractional;
//     let is_large_circle = (cell_index_integral.x % 5.0 == 0.0 && cell_index_integral.y % 5.0 == 0.0);
//     let large_circle_radius = circle_radius * 5.0; // Radius of the larger circles

//     // Make every 25th circle even larger
//     let is_largest_circle = (cell_index_integral.x % 25.0 == 0.0 && cell_index_integral.y % 25.0 == 0.0);
//     let largest_circle_radius = circle_radius * 25.0; // Radius of the largest circles

//     let actual_radius = select(circle_radius, select(large_circle_radius, largest_circle_radius, is_largest_circle), is_large_circle);

//     let in_circle = smoothstep(actual_radius, actual_radius - 0.005, dist_from_center);
//     return in_circle;
// }

// fn circle_pattern(uv: vec2<f32>, base_cell_size: vec2<f32>, base_circle_radius: f32) -> f32 {
//     let power: f32 = 5.0; // Define your power here
//     var in_circle: f32 = 0.0;
//     for (var i: i32 = 1; i < 3; i = i + 1) {
//         let scale: f32 = pow(power, f32(3 - i));
//         let cell_size: vec2<f32> = base_cell_size / (scale * zoom);
//         let circle_radius: f32 = base_circle_radius / (scale * zoom);

//         let cell_uv = uv / cell_size;
//         let cell_index = floor(cell_uv);
//         let cell_center = fract(cell_uv) - 0.5;
//         let dist_from_center = length(cell_center * 2.0); // Scale for proper aspect ratio in cell space

//         // Make every 5th circle larger
//         let cell_index_fractional = fract(cell_index);
//         let cell_index_integral = cell_index - cell_index_fractional;
//         let is_large_circle = (cell_index_integral.x % 4.0 == 0.0 && cell_index_integral.y % 4.0 == 0.0);
//         let large_circle_radius = 0.2 / (scale * zoom); // Radius of the larger circles

//         let actual_radius = select(circle_radius, large_circle_radius, is_large_circle);

//         in_circle = max(in_circle, smoothstep(actual_radius, actual_radius - 0.005, dist_from_center));
//     }
//     return in_circle;
// }

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv;
    let circle_radius = 0.075; // Radius of the circles in the grid pattern
    let circle_value = circle_pattern(uv, grid_cell_size, circle_radius);
    let base_color = vec4<f32>(0.0, 0.0, 0.0, 0.0); // Dark background color
    let final_color = mix(base_color, color, circle_value);

    return final_color;
}