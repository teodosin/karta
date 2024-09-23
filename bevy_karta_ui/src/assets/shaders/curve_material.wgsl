// Shader taken from https://github.com/johnbchron/neutron
// Credit belongs to the author, permission pending. 
// Should permission be denied, this shader will be removed.

#import bevy_sprite::mesh2d_functions as mesh_functions;
#import bevy_sprite::mesh2d_view_bindings::view;

#import "shaders/sdfs.wgsl"::round_box_sdf;

#import "shaders/cubic_bezier_sdf.wgsl"::cubic_bezier_sdf;

struct Vertex {
  @builtin(vertex_index) vertex_index: u32,
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

struct CurveMaterial {
  point_a: vec2<f32>,
  point_b: vec2<f32>,
  point_c: vec2<f32>,
  point_d: vec2<f32>,
  color: vec4<f32>,
  width: f32,
};

@group(2) @binding(0) var<uniform> material: CurveMaterial;

@vertex
fn vertex(
  in: Vertex
) -> VertexOutput {
  var out: VertexOutput;
  out.uv = in.uv;

  var model = mesh_functions::get_world_from_local(in.instance_index);

  let aabb_min = min(min(material.point_a, material.point_b), min(material.point_c, material.point_d)) - (material.width * 2.0);
  let aabb_max = max(max(material.point_a, material.point_b), max(material.point_c, material.point_d)) + (material.width * 2.0);
  var aabb_vertices = array<vec3<f32>, 4>(
    vec3(aabb_min, in.position.z),
    vec3(aabb_min.x, aabb_max.y, in.position.z),
    vec3(aabb_max, in.position.z),
    vec3(aabb_max.x, aabb_min.y, in.position.z),
  );
  let vertex_position = aabb_vertices[in.vertex_index];
  out.vertex_position = vertex_position;

  out.world_position = mesh_functions::mesh2d_position_local_to_world(
    model,
    vec4<f32>(vertex_position, 1.0)
  );
  // out.world_position = vec4(vertex_position, 1.0);
  out.position = mesh_functions::mesh2d_position_world_to_clip(out.world_position);
  out.world_normal = mesh_functions::mesh2d_normal_local_to_world(in.normal, in.instance_index);
  out.world_normal = out.world_normal * 2.0;

  return out;
}

@fragment
fn fragment(
  in: VertexOutput,
) -> @location(0) vec4<f32> {
  let p = in.vertex_position.xy;

  var distance: f32 = cubic_bezier_sdf(
    material.point_a,
    material.point_b,
    material.point_c,
    material.point_d,
    p
  );
  distance = abs(distance) - material.width;

  // return vec4(1.0, 1.0, 1.0, 1.0);

  return mix(
    material.color,
    vec4(material.color.xyz, 0.0),
    smoothstep(0.0, 0.02, distance)
  );
}