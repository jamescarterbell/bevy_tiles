use std::any::TypeId;

use bevy::{ecs::{entity::Entity, world::EntityWorldMut}, sprite_render::{TileData, TilemapChunkTileData}};

use crate::{chunks::ChunkTypes, queries::TileComponent};

unsafe impl TileComponent for TileData {
    #[inline]
    fn insert_tile_into_chunk<const N: usize>(
        self,
        _map_id: Entity,
        mut chunk: EntityWorldMut<'_>,
        chunk_size: usize,
        _tile_c: [i32; N],
        tile_i: usize,
    ) -> Option<Self> {
        match chunk.get_mut::<TilemapChunkTileData>() {
            Some(data) => data,
            None => {
                chunk
                    .get_mut::<ChunkTypes>()
                    .unwrap()
                    .0
                    .insert(TypeId::of::<Self>());
                let chunk = chunk.insert(TilemapChunkTileData(
                    Vec::with_capacity(chunk_size.pow(N.try_into().unwrap())),
                ));
                chunk.get_mut::<TilemapChunkTileData>().unwrap()
            }
        }.get_mut(tile_i).map(|old| std::mem::replace(old, Some(self))).flatten()
    }

    #[inline]
    fn take_tile_from_chunk(chunk: &mut EntityWorldMut<'_>, tile_i: usize) -> Option<Self> {
        let mut location = chunk.get_mut::<TilemapChunkTileData>()?;
        let removed = location.get_mut(tile_i).map(|old| std::mem::replace(old, None)).flatten()?;
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
        _map_id: Entity,
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
                let chunk = chunk.insert(TilemapChunkTileData(
                    Vec::with_capacity(chunk_size.pow(N.try_into().unwrap())),
                ));
                chunk.get_mut::<TilemapChunkTileData>().unwrap()
            }
        };
        let mut old_tiles = Vec::new();
        for ((_, tile_i), tile) in tile_info.zip(tiles) {
            if let Some(old) = location.get_mut(tile_i).map(|old| std::mem::replace(old, Some(tile))).flatten() {
                old_tiles.push(old)
            }
        }
        old_tiles.into_iter()
    }
}