#import bevy_sprite::mesh2d_vertex_output::VertexOutput;

@group(1) @binding(4) var<uniform> color: vec4<f32>;


@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv;
    let base_color = vec4<f32>(color.x, color.y, uv.x * color.z, 1.0); // Dark background color
    let final_color = mix(base_color, color);

    return color;
}