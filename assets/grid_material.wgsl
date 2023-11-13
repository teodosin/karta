#import bevy_sprite::mesh2d_vertex_output::VertexOutput;
#import bevy_sprite::mesh2d_fragment_output::FragmentInput;
@group(1) @binding(0) var<uniform> material: GridMaterial;

struct GridMaterial {
    color: Color,
}

@fragment
fn fragment(
    input: FragmenInput,
) ->  @location(0) vec4<f32> {
    let final_color: vec4<f32> = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    return final_color;
}