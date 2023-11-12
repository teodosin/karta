#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

@group(1) @binding(100) var<uniform> grid_cell_size: f32;
@group(1) @binding(101) var<uniform> grid_cell_count: u32;
@group(1) @binding(102) var<uniform> grid_color: vec4<f32>;

fn sample_grid(
    uv: vec2<f32>
) -> f32 {
    // Allows for further subdividing between UV coordinates
    let grid_subdivisions = 10;
    let grid_subdivisions_f32: vec2<f32> = vec2<f32>(f32(grid_subdivisions.x) + 1.0, f32(grid_subdivisions.y) + 1.0);
    var multi_uv = uv * grid_subdivisions_f32;

    // Make sure line width is between 0.0 and 1.0
    // let line_widths = saturate(grid_line_widths);
    let line_widths = vec2<f32>(0.5, 0.5);

    // difference of UV values between adjacent screen fragments
    let uv_ddxy = vec4<f32>(dpdx(multi_uv), dpdy(multi_uv));

    // some distance calculation eventually used in antialiasing
    let uv_deriv = vec2<f32>(length(uv_ddxy.xz), length(uv_ddxy.yw));

    // if the line_width is more than half the space provided for drawing it,
    // it's really the background then isn't it?
    let invert_line = line_widths > 0.5;

    // select the appropriate line_width based on how large it is
    let target_width = select(line_widths, 1.0 - line_widths, invert_line);

    // we want to draw at least the size of the derivative calculation, and at most
    // half the available space to draw the line
    let draw_width = clamp(target_width, uv_deriv, vec2<f32>(0.5, 0.5));

    // scale the derivative for antialiasing
    let line_aa = uv_deriv * 1.5;

    // these steps are magical
    var grid_uv = abs(fract(multi_uv) * 2.0 - 1.0);
    grid_uv = select(1.0 - grid_uv, grid_uv, invert_line);
    var grid2 = smoothstep(draw_width + line_aa, draw_width - line_aa, grid_uv);
    grid2 *= saturate(target_width / draw_width);
    grid2 = mix(grid2, target_width, saturate(uv_deriv * 2.0 - 1.0));
    grid2 = select(grid2, 1.0 - grid2, invert_line);

    // mix the x and y value to draw it if either x or y needs drawing
    return mix(grid2.x, 1.0, grid2.y);
}

@fragment
fn fragment(
    in: VertexOutput,
    // @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    let grid_mix = sample_grid(in.uv);
    let final_color = mix(vec4<f32>(1.0, 1.0, 1.0, 1.0), grid_color, grid_mix * grid_color[3]);

    let final_color = grid_color;
    return final_color;
}