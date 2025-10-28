use std::{any::TypeId, iter::repeat_n};

use bevy::{
    asset::Handle,
    ecs::{component::Component, entity::Entity, world::EntityWorldMut},
    image::Image,
    math::UVec2,
    sprite_render::{AlphaMode2d, TileData, TilemapChunk, TilemapChunkTileData},
};

use crate::{chunks::ChunkTypes, queries::TileComponent};

/// Maps to [`bevy::sprite_render::TilemapChunk`] on each chunk.
#[derive(Component, Clone, Debug)]
pub struct TilemapRenderingInfo {
    /// Maps to [`bevy::sprite_render::TilemapChunk::tile_display_size`]
    pub tile_display_size: UVec2,
    /// Maps to [`bevy::sprite_render::TilemapChunk::tileset`]
    pub tileset: Handle<Image>,
    /// Maps to [`bevy::sprite_render::TilemapChunk::alpha_mode`]
    pub alpha_mode: AlphaMode2d,
}

unsafe impl TileComponent for TileData {
    #[inline]
    fn insert_tile_into_chunk<const N: usize>(
        self,
        map_id: Entity,
        mut chunk: EntityWorldMut<'_>,
        chunk_size: usize,
        _tile_c: [i32; N],
        mut tile_i: usize,
    ) -> Option<Self> {
        // Bevy lays out chunks top to bottom
        tile_i = (chunk_size - tile_i / chunk_size - 1) * chunk_size + tile_i % chunk_size;
        match chunk.get_mut::<TilemapChunkTileData>() {
            Some(data) => data,
            None => {
                chunk
                    .get_mut::<ChunkTypes>()
                    .unwrap()
                    .0
                    .insert(TypeId::of::<Self>());

                let rendering_info = chunk
                    .world_scope(|world|
                        world
                            .query::<Option<&TilemapRenderingInfo>>()
                            .get(world, map_id)
                            .expect("Chunk's parent map not found")
                            .cloned()
                    );

                if let Some(rendering_info) = rendering_info {
                    let rendering_info = TilemapChunk {
                        chunk_size: UVec2::new(chunk_size as u32, chunk_size as u32),
                        tile_display_size: rendering_info.tile_display_size,
                        tileset: rendering_info.tileset.clone(),
                        alpha_mode: rendering_info.alpha_mode,
                    };
                    chunk.insert((
                        TilemapChunkTileData(repeat_n(None, chunk_size.pow(N.try_into().unwrap())).collect()),
                        rendering_info,
                    ));
                } else {
                    TilemapChunkTileData(repeat_n(None, chunk_size.pow(N.try_into().unwrap())).collect());
                }
                chunk.get_mut::<TilemapChunkTileData>().unwrap()
            }
        }
        .get_mut(tile_i)
        .map(|old| std::mem::replace(old, Some(self)))
        .flatten()
    }

    #[inline]
    fn take_tile_from_chunk(chunk: &mut EntityWorldMut<'_>, tile_i: usize) -> Option<Self> {
        let mut location = chunk.get_mut::<TilemapChunkTileData>()?;
        let removed = location
            .get_mut(tile_i)
            .map(|old| std::mem::replace(old, None))
            .flatten()?;
        if location.0.len() == 0 {
            chunk
                .get_mut::<ChunkTypes>()
                .unwrap()
                .0
                .remove(&TypeId::of::<Self>());
            chunk.remove::<TilemapChunkTileData>();
        }
        Some(removed)
    }

    #[inline]
    fn insert_tile_batch_into_chunk<const N: usize>(
        tiles: impl Iterator<Item = Self>,
        map_id: Entity,
        mut chunk: EntityWorldMut<'_>,
        chunk_size: usize,
        tile_info: impl Iterator<Item = ([i32; N], usize)>,
    ) -> impl Iterator<Item = Self> {
        let mut location = match chunk.get_mut::<TilemapChunkTileData>() {
            Some(data) => data,
            None => {
                chunk
                    .get_mut::<ChunkTypes>()
                    .unwrap()
                    .0
                    .insert(TypeId::of::<Self>());

                let rendering_info = chunk
                    .world_scope(|world|
                        world
                            .query::<Option<&TilemapRenderingInfo>>()
                            .get(world, map_id)
                            .expect("Chunk's parent map not found")
                            .cloned()
                    );

                if let Some(rendering_info) = rendering_info {
                    let rendering_info = TilemapChunk {
                        chunk_size: UVec2::new(chunk_size as u32, chunk_size as u32),
                        tile_display_size: rendering_info.tile_display_size,
                        tileset: rendering_info.tileset.clone(),
                        alpha_mode: rendering_info.alpha_mode,
                    };
                    chunk.insert((
                        TilemapChunkTileData(repeat_n(None, chunk_size.pow(N.try_into().unwrap())).collect()),
                        rendering_info,
                    ));
                } else {
                    TilemapChunkTileData(repeat_n(None, chunk_size.pow(N.try_into().unwrap())).collect());
                }

                chunk.get_mut::<TilemapChunkTileData>().unwrap()
            }
        };
        let mut old_tiles = Vec::new();
        for ((_, mut tile_i), tile) in tile_info.zip(tiles) {
            tile_i = (chunk_size - tile_i / chunk_size - 1) * chunk_size + tile_i % chunk_size;
            if let Some(old) = location
                .get_mut(tile_i)
                .map(|old| std::mem::replace(old, Some(tile)))
                .flatten()
            {
                old_tiles.push(old)
            }
        }
        old_tiles.into_iter()
    }
}
