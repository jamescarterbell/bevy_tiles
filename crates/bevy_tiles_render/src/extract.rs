use bevy::{
    ecs::{
        entity::Entity,
        query::{Changed, With},
        system::{Commands, Query, ResMut},
    },
    render::Extract,
    transform::components::GlobalTransform,
    utils::hashbrown::HashMap,
};
use bevy_tiles::{
    chunks::{Chunk, ChunkCoord, InMap},
    maps::TileMap,
    tiles::{InChunk, TileIndex},
};
use crossbeam::queue::ArrayQueue;

use crate::{
    chunk::internal::{ChunkUniforms, SavedChunks},
    maps::{
        internal::{MapChunks, MapInfo, SavedMaps},
        TileGridSize, TileMapRenderer, TileSize,
    },
};

pub fn extract_chunks(
    mut commands: Commands,
    mut saved_maps: ResMut<SavedMaps>,
    mut saved_chunks: ResMut<SavedChunks>,
    maps: Extract<
        Query<(
            Entity,
            &TileMap,
            &TileMapRenderer,
            Option<&GlobalTransform>,
            Option<&TileSize>,
            Option<&TileGridSize>,
        )>,
    >,
    changed_maps: Extract<
        Query<
            (),
            (
                Changed<TileMap>,
                Changed<TileMapRenderer>,
                Changed<GlobalTransform>,
                Changed<TileSize>,
                Changed<TileGridSize>,
            ),
        >,
    >,
    chunks: Extract<Query<(Entity, &InMap, &Chunk, &ChunkCoord)>>,
    changed_chunks: Extract<Query<(), (Changed<InMap>, Changed<Chunk>, Changed<ChunkCoord>)>>,
    tiles: Extract<Query<(), With<TileIndex>>>,
    changed_tiles: Extract<Query<&InChunk, Changed<TileIndex>>>,
) {
    let maps_iter = maps.iter();
    let mut extracted_maps = Vec::with_capacity(maps_iter.len());
    let mut map_chunks: HashMap<_, _> =
        HashMap::<Entity, MapChunks>::with_capacity(maps_iter.len());

    for (map_id, map, renderer, transform, tile_size, grid_size) in maps_iter {
        map_chunks.insert(map_id, MapChunks::default());
        if let Some(saved_map) = saved_maps.remove(&map_id) {
            if !changed_maps.contains(map_id) {
                extracted_maps.push(saved_map);
                continue;
            }
        }
        let transform = transform.cloned().unwrap_or_default();
        let tile_size = tile_size.cloned().unwrap_or_default();
        let grid_size = grid_size.cloned().unwrap_or_default();
        extracted_maps.push((
            map_id,
            MapInfo {
                chunk_size: map.chunk_size as u32,
                tile_map_renderer: renderer.clone(),
                tile_size,
                grid_size,
                transform,
            },
        ));
    }
    commands.insert_or_spawn_batch(extracted_maps);

    let chunks_len = chunks.iter().len();
    if chunks_len == 0 {
        return;
    }
    let extracted_chunks = ArrayQueue::new(chunks_len);
    let extracted_saved_chunks = ArrayQueue::new(chunks_len);
    let chunk_edges = ArrayQueue::new(chunks_len);

    changed_tiles.iter().for_each(|in_chunk| {
        saved_chunks.remove(&in_chunk.get());
    });

    chunks
        .par_iter()
        .for_each(|(chunk_id, in_map, chunk, chunk_coord)| {
            map_chunks.get(&in_map.get()).unwrap().push(chunk_id);
            chunk_edges.push((chunk_id, in_map.clone()));

            // TODO: Check if it's changed
            if let Some(chunk) = saved_chunks.remove(&chunk_id) {
                if !changed_chunks.contains(chunk_id) {
                    extracted_saved_chunks.push(chunk);
                    return;
                }
            }

            let mut extracted_tile_instances = Vec::with_capacity(chunk.total_size());

            for tile in chunk.get_tiles() {
                if tile.and_then(|tile_id| tiles.get(tile_id).ok()).is_some() {
                    extracted_tile_instances.push(1);
                } else {
                    extracted_tile_instances.push(0);
                }
            }

            extracted_chunks
                .push((
                    chunk_id,
                    ChunkUniforms {
                        chunk_coord: *chunk_coord,
                        tile_instances: Some(extracted_tile_instances),
                    },
                ))
                .expect("Failed to extract chunk: {:?}");
        });

    commands.insert_or_spawn_batch(extracted_saved_chunks);
    commands.insert_or_spawn_batch(extracted_chunks);
    commands.insert_or_spawn_batch(map_chunks);
}
