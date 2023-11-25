use std::marker::PhantomData;

use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::system::{
        lifetimeless::{Read, SQuery, SRes},
        SystemParamItem,
    },
    math::UVec4,
    prelude::Handle,
    render::{
        mesh::GpuBufferInfo,
        render_phase::{RenderCommand, RenderCommandResult, TrackedRenderPass},
        render_resource::PipelineCache,
        view::ViewUniformOffset,
    },
};

use crate::map::TilemapId;
use crate::TilemapTexture;

use super::{
    chunk::{ChunkId, RenderChunk2dStorage, TilemapUniformData},
    material::{MaterialTilemap, RenderMaterialsTilemap},
    prepare::MeshUniform,
    queue::{ImageBindGroups, TilemapViewBindGroup, TransformBindGroup},
    DynamicUniformIndex,
};

pub struct SetMeshViewBindGroup<const I: usize>;
impl<const I: usize> RenderCommand<Transparent2d> for SetMeshViewBindGroup<I> {
    type Param = ();
    type ViewWorldQuery = (Read<ViewUniformOffset>, Read<TilemapViewBindGroup>);
    type ItemWorldQuery = ();
    #[inline]
    fn render<'w>(
        _item: &Transparent2d,
        (view_uniform, pbr_view_bind_group): (&'w ViewUniformOffset, &'w TilemapViewBindGroup),
        _entity: (),
        _param: (),
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(I, &pbr_view_bind_group.value, &[view_uniform.offset]);

        RenderCommandResult::Success
    }
}

pub struct SetTransformBindGroup<const I: usize>;
impl<const I: usize> RenderCommand<Transparent2d> for SetTransformBindGroup<I> {
    type Param = SRes<TransformBindGroup>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = (
        Read<DynamicUniformIndex<MeshUniform>>,
        Read<DynamicUniformIndex<TilemapUniformData>>,
    );
    #[inline]
    fn render<'w>(
        _item: &Transparent2d,
        _view: (),
        (transform_index, tilemap_index): (
            &'w DynamicUniformIndex<MeshUniform>,
            &'w DynamicUniformIndex<TilemapUniformData>,
        ),
        transform_bind_group: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(
            I,
            &transform_bind_group.into_inner().value,
            &[transform_index.index(), tilemap_index.index()],
        );

        RenderCommandResult::Success
    }
}

pub struct SetTextureBindGroup<const I: usize>;
impl<const I: usize> RenderCommand<Transparent2d> for SetTextureBindGroup<I> {
    type Param = SRes<ImageBindGroups>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = Read<TilemapTexture>;
    #[inline]
    fn render<'w>(
        _item: &Transparent2d,
        _view: (),
        texture: &'w TilemapTexture,
        image_bind_groups: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let bind_group = image_bind_groups.into_inner().values.get(texture).unwrap();
        pass.set_bind_group(I, bind_group, &[]);

        RenderCommandResult::Success
    }
}

pub struct SetItemPipeline;
impl RenderCommand<Transparent2d> for SetItemPipeline {
    type Param = SRes<PipelineCache>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = ();
    #[inline]
    fn render<'w>(
        item: &Transparent2d,
        _view: (),
        _entity: (),
        pipeline_cache: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Some(pipeline) = pipeline_cache
            .into_inner()
            .get_render_pipeline(item.pipeline)
        {
            pass.set_render_pipeline(pipeline);
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Failure
        }
    }
}

pub type DrawTilemap = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetTransformBindGroup<1>,
    SetTextureBindGroup<2>,
    DrawMesh,
);

pub type DrawTilemapMaterial<M> = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetTransformBindGroup<1>,
    SetTextureBindGroup<2>,
    SetMaterialBindGroup<M, 3>,
    DrawMesh,
);

pub struct SetMaterialBindGroup<M: MaterialTilemap, const I: usize>(PhantomData<M>);
impl<M: MaterialTilemap, const I: usize> RenderCommand<Transparent2d>
    for SetMaterialBindGroup<M, I>
{
    type Param = (SRes<RenderMaterialsTilemap<M>>, SQuery<&'static Handle<M>>);
    type ViewWorldQuery = ();
    type ItemWorldQuery = Read<TilemapId>;
    #[inline]
    fn render<'w>(
        _item: &Transparent2d,
        _view: (),
        id: &'w TilemapId,
        (material_bind_groups, material_handles): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Ok(material_handle) = material_handles.get(id.0) {
            let bind_group = material_bind_groups
                .into_inner()
                .get(material_handle)
                .unwrap();
            pass.set_bind_group(I, &bind_group.bind_group, &[]);
        }

        RenderCommandResult::Success
    }
}

pub struct DrawMesh;
impl RenderCommand<Transparent2d> for DrawMesh {
    type Param = SRes<RenderChunk2dStorage>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = (Read<ChunkId>, Read<TilemapId>);
    #[inline]
    fn render<'w>(
        _item: &Transparent2d,
        _view: (),
        (chunk_id, tilemap_id): (&'w ChunkId, &'w TilemapId),
        chunk_storage: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Some(chunk) = chunk_storage.into_inner().get(&UVec4::new(
            chunk_id.0.x,
            chunk_id.0.y,
            chunk_id.0.z,
            tilemap_id.0.index(),
        )) {
            if let Some(gpu_mesh) = &chunk.gpu_mesh {
                pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                match &gpu_mesh.buffer_info {
                    GpuBufferInfo::Indexed {
                        buffer,
                        index_format,
                        count,
                    } => {
                        pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                        pass.draw_indexed(0..*count, 0, 0..1);
                    }
                    GpuBufferInfo::NonIndexed {} => {
                        pass.draw(0..gpu_mesh.vertex_count, 0..1);
                    }
                }
            }
        }

        RenderCommandResult::Success
    }
}
