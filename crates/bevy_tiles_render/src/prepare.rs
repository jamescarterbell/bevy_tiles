use bevy::{
    ecs::{
        entity::Entity,
        system::{Commands, ParallelCommands, Query, Res},
    },
    log::debug,
    render::{
        render_resource::CommandEncoderDescriptor,
        renderer::{RenderDevice, RenderQueue},
    },
    utils::hashbrown::HashMap,
};
use bevy_tiles::chunks::ChunkCoord;
use crossbeam::queue::ArrayQueue;

use crate::{
    bindings::{ChunkBatchBindGroups, ChunkBatchBuffer, ChunkBuffer, MapBatchBuffer},
    chunk::internal::{BatchSize, ChunkBatch, ChunkUniforms},
    maps::internal::MapInfo,
    pipeline::TilesChunkPipeline,
};

// Write individual chunk data to the GPU before consolidation
pub fn prepare_chunks(
    mut commands: Commands,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    mut chunks: Query<(Entity, &mut ChunkUniforms)>,
) {
    let size = chunks.iter().len();
    if size == 0 {
        return;
    }
    let uniform_buffers = ArrayQueue::new(size);
    chunks
        .par_iter_mut()
        .for_each(|(chunk_id, mut chunk_uniform)| {
            let mut buffer = ChunkBuffer::new(&mut chunk_uniform);
            buffer.write_buffer(&device, &queue);
            let _ = uniform_buffers.push((chunk_id, buffer));
        });
    commands.insert_or_spawn_batch(uniform_buffers);
}

// Consolidate individual chunk information into the batch entities
pub fn prepare_chunk_batch(
    mut commands: Commands,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    chunks: Query<(&ChunkBatch, &ChunkBuffer)>,
    chunk_batches: Query<(Entity, &BatchSize, &MapInfo)>,
) {
    let batch_iter = chunk_batches.iter();
    let batch_count = batch_iter.len();

    let mut instance_indices = HashMap::with_capacity(batch_count);
    let mut chunk_batch_buffers = HashMap::with_capacity(batch_count);
    let mut global_uniform_buffers = Vec::with_capacity(batch_count);

    if batch_iter.len() == 0 {
        return;
    }

    for (batch_id, batch_size, map_info) in batch_iter {
        debug!(
            "Preparing batch {:?} with size {:?}",
            batch_id, **batch_size
        );

        // Create all our instance buffers before we start iterating over chunks
        instance_indices.insert(batch_id, 0);
        chunk_batch_buffers.insert(
            batch_id,
            ChunkBatchBuffer::with_size_no_default_values(
                **batch_size as usize,
                map_info.chunk_size as usize,
                &device,
            ),
        );

        // Create all our global uniforms for the batches
        let mut map_buffers = MapBatchBuffer::new(map_info);

        map_buffers.write_buffer(&device, &queue);

        global_uniform_buffers.push((batch_id, (map_buffers)));
    }

    let mut command_encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("bevy_tiles_render::batch_buffer_copies"),
    });

    for (batch_id, chunk_buffer) in chunks.iter() {
        let chunk_batch_buffer = chunk_batch_buffers.get_mut(&**batch_id).unwrap();
        chunk_batch_buffer.push(&mut command_encoder, chunk_buffer);
    }

    for (_, buffer) in chunk_batch_buffers.iter_mut() {
        buffer.write_buffer(&device, &queue)
    }

    queue.submit([command_encoder.finish()]);

    commands.insert_or_spawn_batch(global_uniform_buffers);
    commands.insert_or_spawn_batch(chunk_batch_buffers);
}

pub fn create_bind_groups(
    mut commands: Commands,
    device: Res<RenderDevice>,
    chunk_pipeline: Res<TilesChunkPipeline>,
    chunk_batches: Query<(Entity, &MapBatchBuffer, &ChunkBatchBuffer)>,
) {
    // Create bind groups
    debug!(
        "Creating bind group for {} batches",
        chunk_batches.iter().len()
    );
    for (batch_id, map_buffers, chunk_offsets) in chunk_batches.iter() {
        let map_bind_group = device.create_bind_group(
            "batch_map_bind_group",
            &chunk_pipeline.chunk_batch_bind_groups.map_layouts,
            &map_buffers.bindings(),
        );

        let chunk_bind_group = device.create_bind_group(
            "batch_chunk_bind_group",
            &chunk_pipeline.chunk_batch_bind_groups.chunk_layouts,
            &chunk_offsets.bindings(),
        );

        debug!("Adding bind groups to batch {:?}", batch_id);
        commands.entity(batch_id).insert(ChunkBatchBindGroups {
            map_bind_group,
            chunk_bind_group,
        });
    }
}
