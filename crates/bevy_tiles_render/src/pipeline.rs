use bevy::{
    ecs::{
        system::Resource,
        world::{FromWorld, World},
    },
    render::{
        render_resource::{
            BindGroupLayout, BlendState, ColorTargetState, ColorWrites, Face, FragmentState,
            FrontFace, MultisampleState, PolygonMode, PrimitiveState, PrimitiveTopology,
            RenderPipelineDescriptor, SpecializedRenderPipeline, TextureFormat, VertexBufferLayout,
            VertexFormat, VertexState, VertexStepMode,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
        view::ViewTarget,
    },
    sprite::{Mesh2dPipeline, Mesh2dPipelineKey},
};

use crate::{
    bindings::{ChunkBatchBindGroupLayouts, MapTransformUniform},
    TILES_FRAG, TILES_VERT,
};

#[derive(Resource)]
pub struct TilesChunkPipeline {
    pub mesh2d_pipeline: Mesh2dPipeline,
    pub chunk_batch_bind_groups: ChunkBatchBindGroupLayouts,
}

impl FromWorld for TilesChunkPipeline {
    fn from_world(world: &mut World) -> Self {
        Self {
            mesh2d_pipeline: Mesh2dPipeline::from_world(world),
            chunk_batch_bind_groups: ChunkBatchBindGroupLayouts::from_world(world),
        }
    }
}

impl SpecializedRenderPipeline for TilesChunkPipeline {
    type Key = Mesh2dPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
    ) -> bevy::render::render_resource::RenderPipelineDescriptor {
        let format = match key.contains(Mesh2dPipelineKey::HDR) {
            true => ViewTarget::TEXTURE_FORMAT_HDR,
            false => TextureFormat::bevy_default(),
        };

        let shader_defs = Vec::new();

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: TILES_VERT,
                entry_point: "vs_main".into(),
                shader_defs: shader_defs.clone(),
                // We generate clip space triangles on the fly based on the implicit index buffer
                buffers: vec![],
            },
            fragment: Some(FragmentState {
                // Use our custom shader
                shader: TILES_FRAG,
                shader_defs,
                entry_point: "fs_main".into(),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            // Use the two standard uniforms for 2d meshes
            layout: vec![
                // Bind group 0 is the view uniform
                self.mesh2d_pipeline.view_layout.clone(),
                // Bind group 1 are the map components
                self.chunk_batch_bind_groups.map_layouts.clone(),
                // Bind group 2 are the chunk components
                self.chunk_batch_bind_groups.chunk_layouts.clone(),
            ],
            push_constant_ranges: Vec::new(),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some("tiles_pipeline".into()),
        }
    }
}
