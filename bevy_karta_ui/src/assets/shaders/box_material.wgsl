// Shader taken from https://github.com/johnbchron/neutron
// Credit belongs to the author, permission pending. 
// Should permission be denied, this shader will be removed.

#import bevy_sprite::mesh2d_functions as mesh_functions
#import bevy_sprite::mesh2d_view_bindings

#import "shaders/sdfs.wgsl"::round_box_sdf

struct Vertex {
  @builtin(instance_index) instance_index: u32,
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
};

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) world_position: vec4<f32>,
  @location(1) world_normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) vertex_position: vec3<f32>,
}

struct NodeMaterial {
  color: vec4<f32>,
  size: vec2<f32>,
  border_width: f32,
  border_color: vec4<f32>,
  radius: f32,
};

@group(2) @binding(0) var<uniform> material: NodeMaterial;

@vertex
fn vertex(
  in: Vertex,
) -> VertexOutput {
  var out: VertexOutput;
  out.uv = in.uv;

  var model = mesh_functions::get_world_from_local(in.instance_index);
  let vertex_position = in.position * vec3(material.size + material.border_width * 2.0, in.position.z);

  out.vertex_position = vertex_position;
  out.world_position = mesh_functions::mesh_position_local_to_world(
    model,
    vec4<f32>(vertex_position, 1.0)
  );
  out.position = mesh_functions::mesh_position_world_to_clip(out.world_position);
  out.world_normal = mesh_functions::mesh_normal_local_to_world(in.normal, in.instance_index);

  return out;
}

@fragment
fn fragment(
  in: VertexOutput,
) -> @location(0) vec4<f32> {
  let p = in.vertex_position.xy;
  let distance = round_box_sdf(material.size / 2.0, material.radius, p);

  if distance < 0.0 {
    return material.color;
  } else if distance < material.border_width {
    return material.border_color;
  } else {
    return vec4(0.0);
  }
}