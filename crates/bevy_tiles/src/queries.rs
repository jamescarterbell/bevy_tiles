use bevy::{
    ecs::{entity::Entity, query::QueryData},
    prelude::EntityWorldMut,
};

use crate::chunks::ChunkData;

/// Marks a data type as.
pub trait TileQueryData {
    /// The item returned from a tile query.
    type Item<'w, 's>;
    /// The component on the chunk tile data is queried from.
    type Source: QueryData;

    /// Get tile data from a chunk.
    fn get<'w, 's, 'i: 's>(
        source: <<Self as TileQueryData>::Source as QueryData>::Item<'w, 's>,
        index: usize,
    ) -> Option<Self::Item<'w, 'i>>;
}

/// Mark type as usable in tiles.
pub trait TileData: TileQueryData + Send + Sync {
    /// The readonly variant of the tile data.
    type ReadOnly: ReadOnlyTileData
        + TileQueryData<Source = <<Self as TileQueryData>::Source as QueryData>::ReadOnly>;
}

/// Mark type as usable in readonly tile queries.
/// # Safety
/// Only safe to impl on readonly types.
pub unsafe trait ReadOnlyTileData: TileData<ReadOnly = Self> {}

impl<T: Send + Sync + 'static> TileData for &T {
    type ReadOnly = Self;
}

/// Safety: &T is readonly.
unsafe impl<T: Send + Sync + 'static> ReadOnlyTileData for &T {}

impl<T: Send + Sync + 'static> TileQueryData for &T {
    type Item<'w, 's> = &'w T;

    type Source = &'static ChunkData<T>;

    fn get<'w, 's, 'i: 's>(
        source: <<Self as TileQueryData>::Source as QueryData>::Item<'w, 's>,
        index: usize,
    ) -> Option<Self::Item<'w, 'i>> {
        source.get(index)
    }
}

impl<T: Send + Sync + 'static> TileData for &mut T {
    type ReadOnly = &'static T;
}

impl<T: Send + Sync + 'static> TileQueryData for &mut T {
    type Item<'w, 's> = &'w mut T;

    type Source = &'static mut ChunkData<T>;

    fn get<'w, 's, 'i: 's>(
        source: <<Self as TileQueryData>::Source as QueryData>::Item<'w, 's>,
        index: usize,
    ) -> Option<Self::Item<'w, 'i>> {
        source.into_inner().get_mut(index)
    }
}

/// The tiled version of a component bundle.
/// # Safety
/// Easy to screw this up.
pub unsafe trait TileComponent: Sized + Send + Sync + 'static {
    /// Inserts a bundle and returns all the replaced values.
    fn insert_tile_into_chunk<const N: usize>(
        self,
        map_id: Entity,
        chunk: EntityWorldMut<'_>,
        chunk_size: usize,
        tile_c: [i32; N],
        tile_i: usize,
    ) -> Option<Self>;

    /// Inserts a bundle and returns all the replaced values.
    fn insert_tile_batch_into_chunk<const N: usize>(
        tiles: impl Iterator<Item = Self>,
        map_id: Entity,
        chunk: EntityWorldMut<'_>,
        chunk_size: usize,
        tile_info: impl Iterator<Item = ([i32; N], usize)>,
    ) -> impl Iterator<Item = Self>;

    /// Try to remove a bundle.
    fn take_tile_from_chunk(chunk: &mut EntityWorldMut<'_>, tile_i: usize) -> Option<Self>;
}
