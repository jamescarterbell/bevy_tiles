//! Contains commands given to the Transparent2D render step
//! to draw tiles.
//!
//! In order to batch draw calls of batched chunk draws (used for larger scenes if for lower memory situations)
//! the draw commands consist of copying individual chunk buffers to the various instance buffers before
//! issuing a draw call for a given batch of chunks.
use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::{
        query::ROQueryItem,
        system::{lifetimeless::Read, SystemParamItem},
    },
    log::debug,
    render::render_phase::{
        PhaseItem, RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass,
    },
    sprite::SetMesh2dViewBindGroup,
};

use crate::{bindings::ChunkBatchBindGroups, chunk::internal::BatchSize, maps::internal::MapInfo};

pub type DrawChunks = (
    SetItemPipeline,
    SetMesh2dViewBindGroup<0>,
    SetMapBindGroup<1>,
    SetChunkBindGroup<2>,
    DrawChunkBatch,
);

pub struct SetMapBindGroup<const I: usize>;
impl<const I: usize> RenderCommand<Transparent2d> for SetMapBindGroup<I> {
    type Param = ();

    type ViewQuery = ();

    type ItemQuery = Read<ChunkBatchBindGroups>;

    #[inline]
    fn render<'w>(
        item: &Transparent2d,
        _view: (),
        bind_groups: Option<ROQueryItem<'w, Self::ItemQuery>>,
        _: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        debug!("Setting Chunk Map Level Bind Groups");

        let mut dynamic_offsets: [u32; 1] = Default::default();
        let mut offset_count = 0;
        if let Some(dynamic_offset) = item.dynamic_offset() {
            dynamic_offsets[offset_count] = dynamic_offset.get();
            offset_count += 1;
        }

        let Some(bind_groups) = bind_groups else {
            return RenderCommandResult::Failure;
        };

        pass.set_bind_group(
            I,
            &bind_groups.map_bind_group,
            &dynamic_offsets[..offset_count],
        );
        RenderCommandResult::Success
    }
}

pub struct SetChunkBindGroup<const I: usize>;
impl<const I: usize> RenderCommand<Transparent2d> for SetChunkBindGroup<I> {
    type Param = ();

    type ViewQuery = ();

    type ItemQuery = Read<ChunkBatchBindGroups>;

    #[inline]
    fn render<'w>(
        item: &Transparent2d,
        _view: (),
        bind_groups: Option<ROQueryItem<'w, Self::ItemQuery>>,
        _: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        debug!("Setting Chunk Level Bind Groups");

        let mut dynamic_offsets: [u32; 1] = Default::default();
        let mut offset_count = 0;
        if let Some(dynamic_offset) = item.dynamic_offset() {
            dynamic_offsets[offset_count] = dynamic_offset.get();
            offset_count += 1;
        }

        let Some(bind_groups) = bind_groups else {
            return RenderCommandResult::Failure;
        };

        pass.set_bind_group(
            I,
            &bind_groups.chunk_bind_group,
            &dynamic_offsets[..offset_count],
        );
        RenderCommandResult::Success
    }
}

pub struct DrawChunkBatch;
impl RenderCommand<Transparent2d> for DrawChunkBatch {
    type Param = ();

    type ViewQuery = ();

    type ItemQuery = (Read<MapInfo>, Read<BatchSize>);

    #[inline]
    fn render<'w>(
        item: &Transparent2d,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        itemq: Option<ROQueryItem<'w, Self::ItemQuery>>,
        _: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some((map_info, batch_size)) = itemq else {
            return RenderCommandResult::Failure;
        };

        pass.draw(
            0..(map_info.chunk_size * map_info.chunk_size * 6),
            0..**batch_size,
        );
        RenderCommandResult::Success
    }
}
