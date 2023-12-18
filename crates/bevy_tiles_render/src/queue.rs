use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::{
        entity::Entity,
        query::{Or, With},
        system::{Commands, ParallelCommands, Query, Res, ResMut},
    },
    log::debug,
    render::{
        render_phase::{DrawFunctions, RenderPhase},
        render_resource::{PipelineCache, PrimitiveTopology, SpecializedRenderPipelines},
        renderer::RenderDevice,
        view::{ExtractedView, Msaa},
    },
    sprite::Mesh2dPipelineKey,
    utils::FloatOrd,
};
use bevy_tiles::{chunks::InMap, maps::TileMap};

use crate::{
    bindings::ChunkBuffer,
    chunk::internal::{BatchSize, ChunkBatch, ChunkUniforms},
    draw::DrawChunks,
    maps::internal::{MapChunks, MapInfo},
    pipeline::TilesChunkPipeline,
};

pub fn create_chunk_batches(
    commands: ParallelCommands,
    maps: Query<(&MapInfo, &MapChunks)>,
    chunks: Query<Entity, Or<(With<ChunkUniforms>, With<ChunkBuffer>)>>,
) {
    maps.par_iter().for_each(|(map_info, map_chunks)| {
        commands.command_scope(|mut commands| {
            let max_batch_size = map_info.tile_map_renderer.batch_size;
            let chunk_count = chunks.iter().len();
            let batch_count = chunk_count / max_batch_size as usize
                + if (chunk_count % max_batch_size as usize) > 0 {
                    1
                } else {
                    0
                };

            if batch_count == 0 {
                return;
            }

            let mut batches = Vec::with_capacity(batch_count);
            let mut batched_chunks = Vec::with_capacity(chunk_count);

            let mut batch_size = 0;
            let mut current_batch = ChunkBatch(commands.spawn_empty().id());

            while let Some(chunk_id) = map_chunks.pop() {
                if chunks.get(chunk_id).is_ok() {
                    if batch_size == max_batch_size {
                        batches.push((*current_batch, (BatchSize(batch_size), map_info.clone())));
                        batch_size = 0;
                        current_batch = ChunkBatch(commands.spawn_empty().id());
                    }
                    batched_chunks.push((chunk_id, current_batch.clone()));
                    batch_size += 1;
                }
            }

            if batch_size > 0 {
                batches.push((*current_batch, (BatchSize(batch_size), map_info.clone())));
            }

            commands.insert_or_spawn_batch(batches);
            commands.insert_or_spawn_batch(batched_chunks);
        });
    });
}

pub fn queue_chunks(
    mut commands: Commands,
    mut pipelines: ResMut<SpecializedRenderPipelines<TilesChunkPipeline>>,
    device: Res<RenderDevice>,
    chunk_pipeline: Res<TilesChunkPipeline>,
    pipeline_cache: Res<PipelineCache>,
    msaa: Res<Msaa>,
    transparent_draw_functions: Res<DrawFunctions<Transparent2d>>,
    mut views: Query<(&mut RenderPhase<Transparent2d>, &ExtractedView)>,
    chunk_batches: Query<(Entity, &BatchSize)>,
) {
    for (mut transparent_phase, view) in &mut views {
        let chunk_batch_iter = chunk_batches.iter();
        if chunk_batch_iter.len() == 0 {
            continue;
        }

        let mesh_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples())
            | Mesh2dPipelineKey::from_hdr(view.hdr)
            | Mesh2dPipelineKey::from_primitive_topology(PrimitiveTopology::TriangleList);
        let pipeline_id = pipelines.specialize(&pipeline_cache, &chunk_pipeline, mesh_key);

        let draw_chunks = transparent_draw_functions.read().id::<DrawChunks>();

        for (batch_id, batch_size) in chunk_batch_iter {
            debug!("Queuing draw call for batch: {:?}", batch_id);
            transparent_phase.add(Transparent2d {
                entity: batch_id,
                draw_function: draw_chunks,
                pipeline: pipeline_id,
                sort_key: FloatOrd(0.0),
                // Ignore this, we do our own batching
                batch_range: 0..1,
                dynamic_offset: None,
            });
        }

        debug!(
            "Queued {:?} Chunk Batches for Drawing",
            chunk_batches.iter().len()
        );
    }
}
