use std::any::TypeId;

use bevy::{
    ecs::query::WorldQuery,
    math::{IVec2, IVec3, Vec2, Vec3},
    prelude::{Component, Deref, DerefMut, Entity, EntityWorldMut},
};
use bevy_tiles::{
    chunks::{ChunkData, ChunkTypes},
    queries::{ReadOnlyTileData, TileComponent, TileData, TileDataQuery},
};

#[derive(Deref, DerefMut, Clone, Copy, Debug, PartialEq, Eq)]
/// TileComponent for tracking entities.
pub struct EntityTile(pub Entity);

impl TileData for EntityTile {
    type ReadOnly = Self;
}

/// Safety: Entity is readonly.
unsafe impl ReadOnlyTileData for EntityTile {}

impl TileDataQuery for EntityTile {
    type Item<'a> = EntityTile;

    type Source = &'static ChunkData<EntityTile>;

    fn get<'a>(
        source: <<Self as TileDataQuery>::Source as WorldQuery>::Item<'_>,
        index: usize,
    ) -> Option<Self::Item<'_>> {
        source.get(index).cloned()
    }
}

/// # Safety:
/// Probably safe.
/// MAKE THIS NOT A DEFAULT IMPL
unsafe impl TileComponent for EntityTile {
    fn insert_tile_into_chunk<const N: usize>(
        self,
        mut chunk: EntityWorldMut<'_>,
        chunk_c: [i32; N],
        chunk_size: usize,
        tile_c: [i32; N],
        tile_i: usize,
    ) -> Option<Self> {
        let location = match chunk.get_mut::<ChunkData<Self>>() {
            Some(data) => data,
            None => {
                chunk
                    .get_mut::<ChunkTypes>()
                    .unwrap()
                    .0
                    .insert(TypeId::of::<Self>());
                let chunk = chunk.insert(ChunkData::<Self>::new(
                    chunk_size.pow(N.try_into().unwrap()),
                ));
                chunk.get_mut::<ChunkData<Self>>().unwrap()
            }
        };
        let mut binding = location;
        let res = binding.insert(tile_i, self);

        let chunk_id = chunk.id();

        chunk.world_scope(|world| {
            world.get_entity_mut(*self).unwrap().insert((
                TileIndex(tile_i),
                TileCoord(tile_c),
                InChunk(chunk_id),
            ));
        });

        res
    }

    fn take_tile_from_chunk(chunk: &mut EntityWorldMut<'_>, tile_i: usize) -> Option<Self> {
        let location = chunk.get_mut::<ChunkData<Self>>();
        let mut binding = location?;
        let removed = binding.take(tile_i);
        if binding.get_count() == 0 {
            chunk
                .get_mut::<ChunkTypes>()
                .unwrap()
                .0
                .remove(&TypeId::of::<Self>());
            chunk.remove::<ChunkData<Self>>();
        }
        removed
    }

    fn insert_tile_batch_into_chunk<const N: usize>(
        tiles: impl Iterator<Item = Self>,
        mut chunk: EntityWorldMut<'_>,
        chunk_c: [i32; N],
        chunk_size: usize,
        tile_is: impl Iterator<Item = ([i32; N], usize)>,
    ) -> impl Iterator<Item = Self> {
        let chunk_id = chunk.id();
        let mut chunk_data = match chunk.take::<ChunkData<Self>>() {
            Some(data) => data,
            None => {
                chunk
                    .get_mut::<ChunkTypes>()
                    .unwrap()
                    .0
                    .insert(TypeId::of::<Self>());
                ChunkData::<Self>::new(chunk_size.pow(N.try_into().unwrap()))
            }
        };

        let mut removed = Vec::new();
        for ((tile_c, tile_i), tile) in tile_is.zip(tiles) {
            let res = chunk_data.insert(tile_i, tile);

            chunk.world_scope(|world| {
                world.get_entity_mut(*tile).unwrap().insert((
                    TileIndex(tile_i),
                    TileCoord(tile_c),
                    InChunk(chunk_id),
                ));
            });

            if let Some(res) = res {
                removed.push(res);
            }
        }

        chunk.insert(chunk_data);
        removed.into_iter()
    }
}

/// The index of a tile in a given chunk.
/// # Note:
/// It probably won't break anything to manually copy this
/// to put it on your own entities, but this is only accurate
/// when mutated by the plugin.
#[derive(Component, Clone, Copy, PartialEq, Eq, Deref, Debug)]
pub struct TileIndex(pub(crate) usize);

/// The coordinate of a tile in a given map.
/// # Note:
/// It probably won't break anything to manually copy this
/// to put it on your own entities, but this is only accurate
/// when mutated by the plugin.
#[derive(Component, Deref, Clone, Copy, PartialEq, Eq, Debug)]
pub struct TileCoord<const N: usize>(pub(crate) [i32; N]);

impl From<TileCoord<3>> for IVec3 {
    fn from(value: TileCoord<3>) -> Self {
        value.0.into()
    }
}

impl From<TileCoord<2>> for IVec2 {
    fn from(value: TileCoord<2>) -> Self {
        value.0.into()
    }
}

impl From<TileCoord<3>> for Vec3 {
    fn from(value: TileCoord<3>) -> Self {
        Vec3::new(value[0] as f32, value[1] as f32, value[2] as f32)
    }
}

impl From<TileCoord<2>> for Vec2 {
    fn from(value: TileCoord<2>) -> Self {
        Vec2::new(value[0] as f32, value[1] as f32)
    }
}

/// A relation on tiles that point towards the chunk they are a part of.
#[derive(Component, Deref, Debug)]
pub struct InChunk(pub(crate) Entity);
