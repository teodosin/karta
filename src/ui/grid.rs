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
        app.insert_resource(InfiniteGrid2DSettings::default());
    }

    // fn finish(&self, app: &mut App) {
    //     render_app_builder(app);
    // }
}

#[derive(Resource)]
pub struct InfiniteGrid2DSettings {
    pub cell_size: f32,
    pub cell_count: u32,
    pub color: Color,
}

impl Default for InfiniteGrid2DSettings {
    fn default() -> Self {
        InfiniteGrid2DSettings {
            cell_size: 1.0,
            cell_count: 10,
            color: Color::WHITE,
        }
    }
}