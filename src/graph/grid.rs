use std::borrow::Cow;

use bevy::{
    prelude::*, 
    render::{
        render_resource::{
            ShaderType, BindGroup, DynamicUniformBuffer, PipelineCache, BindGroupLayout, SpecializedRenderPipelines, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ShaderStages, BindingType, BufferBindingType, BufferSize, SpecializedRenderPipeline, RenderPipelineDescriptor, TextureFormat, VertexState, PrimitiveState, PrimitiveTopology, PolygonMode, DepthStencilState, CompareFunction, StencilState, StencilFaceState, DepthBiasState, MultisampleState, FragmentState, ColorTargetState, BlendState, ColorWrites
        }, 
        render_phase::{
            PhaseItem, RenderCommand, RenderCommandResult, RenderPhase, DrawFunctions, SetItemPipeline, AddRenderCommand
        }, view::{VisibleEntities, ExtractedView, ViewTarget}, Extract, renderer::{RenderDevice, RenderQueue}, texture::BevyDefault, RenderApp, Render, RenderSet, self
    }, 
    reflect::TypeUuid, 
    ecs::{
        system::{
            lifetimeless::{
                Read, SRes
            }, SystemParamItem
        }, query::ROQueryItem
    }, core_pipeline::core_3d::Transparent3d, pbr::MeshPipelineKey
};

// Modeled after lib.rs of bevy_infinite_grid

pub struct InfiniteGrid2DPlugin;

impl Plugin for InfiniteGrid2DPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InfiniteGrid2DSettings>();
    }

    fn finish(&self, app: &mut App) {
        render_app_builder(app);
    }
}

#[derive(Resource, Default)]
pub struct InfiniteGrid2DSettings {
    pub color: Color,
}

#[derive(Component, Copy, Clone)]
pub struct InfiniteGrid2D {
    pub color: Color,
}

#[derive(Bundle)]
pub struct InfiniteGrid2DBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub grid: InfiniteGrid2D,
    // pub frustum_intersect: GridFrustumIntersect
    // Maybe not needed for 2d?
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility
}

impl Default for InfiniteGrid2DBundle {
    fn default() -> Self {
        Self {
            transform: Default::default(),
            global_transform: Default::default(),
            grid: InfiniteGrid2D {
                color: Color::rgb(0.0, 0.0, 0.0),
            },
            visibility: Default::default(),
            computed_visibility: Default::default()
        }
    }
}

// Modeled after render/mod.rs or bevy_infinite_grid

const SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 0x9a9e9e9e9e9e9e9e);

static PLANE_RENDER: &str = include_str!("plane_render.wgsl");


#[derive(Component)]
pub struct ExtractedInfiniteGrid2D {
    transform: GlobalTransform,
    grid: InfiniteGrid2D,
}

#[derive(Debug, ShaderType)]
pub struct InfiniteGrid2DUniform {
    offset: Vec2,
    scale: f32,
    color: Vec3,
}

#[derive(Resource, Default)]
struct InfiniteGrid2DUniforms {
    uniforms: DynamicUniformBuffer<InfiniteGrid2DUniform>
}

#[derive(Component)]
struct InfiniteGrid2DUniformOffset {
    offset: u32,
}

#[derive(Resource)]
struct InfiniteGrid2DBindGroup {
    value: BindGroup,
}

struct SetInfiniteGrid2DBindGroup<const I: usize>;

impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetInfiniteGrid2DBindGroup<I> {
    type Param = SRes<InfiniteGrid2DBindGroup>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = Read<InfiniteGrid2DUniformOffset>;

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: ROQueryItem<'w, Self::ViewWorldQuery>,
        offset: ROQueryItem<'w, Self::ItemWorldQuery>,
        bind_group: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(I, &bind_group.into_inner().value, &[offset.offset]);
        RenderCommandResult::Success
    }
}

struct FinishDrawInfiniteGrid;

impl<P: PhaseItem> RenderCommand<P> for FinishDrawInfiniteGrid {
    type Param = ();
    type ViewWorldQuery = ();
    type ItemWorldQuery = ();

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: ROQueryItem<'w, Self::ViewWorldQuery>,
        _entity: ROQueryItem<'w, Self::ItemWorldQuery>,
        _param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.draw(0..4, 0..1);
        RenderCommandResult::Success
    }
}

fn extract_infinite_grids(
    mut commands: Commands,
    grids: Extract<Query<(Entity, &InfiniteGrid2D, &GlobalTransform, &VisibleEntities)>>,
) {
    let extracted: Vec<_> = grids
        .iter()
        .map(|(entity, grid, transform, visible_entities)| {
            (
                entity,
                (
                    ExtractedInfiniteGrid2D {
                        transform: *transform,
                        grid: *grid,
                    },
                    visible_entities.clone(),
                ),
            )
        })
        .collect();
    commands.insert_or_spawn_batch(extracted);
}

fn prepare_infinite_grids(
    mut commands: Commands,
    grids: Query<(Entity, &ExtractedInfiniteGrid2D)>,
    mut uniforms: ResMut<InfiniteGrid2DUniforms>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    uniforms.uniforms.clear();
    for (entity, extracted) in grids.iter() {
        let transform = extracted.transform;
        let t = transform.compute_transform();
        let offset = transform.translation().truncate();
        commands.entity(entity).insert(InfiniteGrid2DUniformOffset {
            offset: uniforms.uniforms.push(InfiniteGrid2DUniform {
                offset,
                scale: t.scale.x,
                color: Vec3::from_slice(&extracted.grid.color.as_rgba_f32())
            }),
        });
    }

    uniforms
        .uniforms
        .write_buffer(&render_device, &render_queue);
}

#[allow(clippy::too_many_arguments)]
fn queue_infinite_grids(
    pipeline_cache: Res<PipelineCache>,
    transparent_draw_functions: Res<DrawFunctions<Transparent3d>>,
    mut commands: Commands,
    uniforms: Res<InfiniteGrid2DUniforms>,
    pipeline: Res<InfiniteGrid2DPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<InfiniteGrid2DPipeline>>,
    render_device: Res<RenderDevice>,
    infinite_grids: Query<&ExtractedInfiniteGrid2D>,
    mut views: Query<(
        &VisibleEntities,
        &mut RenderPhase<Transparent3d>,
        &ExtractedView,
    )>,
    msaa: Res<Msaa>,
) {
    let bind_group = if let Some(binding) = uniforms.uniforms.binding() {
        render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("infinite-grid-bind-group"),
            layout: &pipeline.infinite_grid_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: binding.clone(),
            }],
        })
    } else {
        return;
    };
    commands.insert_resource(InfiniteGrid2DBindGroup { value: bind_group });

    let draw_function_id = transparent_draw_functions
        .read()
        .get_id::<DrawInfiniteGrid>()
        .unwrap();

    for (entities, mut phase, view) in views.iter_mut() {
        let mesh_key = MeshPipelineKey::from_hdr(view.hdr);
        let base_pipeline = pipelines.specialize(
            &pipeline_cache,
            &pipeline,
            GridPipelineKey {
                mesh_key,
                sample_count: msaa.samples(),
            },
        );
        let shadow_pipeline = pipelines.specialize(
            &pipeline_cache,
            &pipeline,
            GridPipelineKey {
                mesh_key,
                sample_count: msaa.samples(),
            },
        );
        for &entity in &entities.entities {
            phase.items.push(Transparent3d {
                pipeline: base_pipeline,
                entity,
                draw_function: draw_function_id,
                distance: f32::NEG_INFINITY,
            });
        }
    }
}

type DrawInfiniteGrid = (
    SetItemPipeline,
    SetInfiniteGrid2DBindGroup<1>,
    FinishDrawInfiniteGrid,
);

#[derive(Resource)]
struct InfiniteGrid2DPipeline {
    infinite_grid_layout: BindGroupLayout,
}

impl FromWorld for InfiniteGrid2DPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let infinite_grid_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("infinite-grid-bind-group-layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: BufferSize::new(InfiniteGrid2DUniform::min_size().into()),
                    },
                    count: None,
                }],
            });



        Self {
            infinite_grid_layout,
        }
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct GridPipelineKey {
    mesh_key: MeshPipelineKey,
    sample_count: u32,
}

impl SpecializedRenderPipeline for InfiniteGrid2DPipeline {
    type Key = GridPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let format = match key.mesh_key.contains(MeshPipelineKey::HDR) {
            true => ViewTarget::TEXTURE_FORMAT_HDR,
            false => TextureFormat::bevy_default(),
        };

        RenderPipelineDescriptor {
            label: Some(Cow::Borrowed("grid-render-pipeline")),
            layout: Vec::new(),
            push_constant_ranges: Vec::new(),
            vertex: VertexState {
                shader: SHADER_HANDLE.typed(),
                shader_defs: vec![],
                entry_point: Cow::Borrowed("vertex"),
                buffers: vec![],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: bevy::render::render_resource::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: CompareFunction::Greater,
                stencil: StencilState {
                    front: StencilFaceState::IGNORE,
                    back: StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            multisample: MultisampleState {
                count: key.sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                shader: SHADER_HANDLE.typed(),
                shader_defs: vec![],
                entry_point: Cow::Borrowed("fragment"),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
        }
    }
}

pub fn render_app_builder(app: &mut App) {
    app.world
        .resource_mut::<Assets<Shader>>()
        .set_untracked(SHADER_HANDLE, Shader::from_wgsl(PLANE_RENDER, file!()));

    let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
        return
    };
    render_app
        .init_resource::<InfiniteGrid2DUniforms>()
        .init_resource::<InfiniteGrid2DPipeline>()
        .init_resource::<SpecializedRenderPipelines<InfiniteGrid2DPipeline>>()
        .add_render_command::<Transparent3d, DrawInfiniteGrid>()
        .add_systems(
            ExtractSchedule,
            (extract_infinite_grids).chain(), // order to minimize move overhead
        )
        .add_systems(
            Render,
            (
                prepare_infinite_grids,
            )
                .in_set(RenderSet::Prepare),
        )
        .add_systems(
            Render,
            (queue_infinite_grids).in_set(RenderSet::Queue),
        );
}