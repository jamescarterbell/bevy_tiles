use bevy::ecs::query::{QueryData, WorldQuery};

use crate::chunks::ChunkData;

/// Marks a data type as.
pub trait TileDataQuery {
    /// The item returned from a tile query.
    type Item<'a>;
    /// The component on the chunk tile data is queried from.
    type Source: QueryData;

    /// Get tile data from a chunk.
    fn get(
        source: <<Self as TileDataQuery>::Source as WorldQuery>::Item<'_>,
        index: usize,
    ) -> Option<Self::Item<'_>>;
}

/// Mark type as usable in tiles.
pub trait TileData: TileDataQuery + Send + Sync {
    /// The readonly variant of the tile data.
    type ReadOnly: ReadOnlyTileData
        + TileDataQuery<Source = <<Self as TileDataQuery>::Source as QueryData>::ReadOnly>;
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

impl<T: Send + Sync + 'static> TileDataQuery for &T {
    type Item<'a> = &'a T;

    type Source = &'static ChunkData<T>;

    fn get<'a>(
        source: <<Self as TileDataQuery>::Source as WorldQuery>::Item<'_>,
        index: usize,
    ) -> Option<Self::Item<'_>> {
        source.get(index)
    }
}

impl<'w, T: Send + Sync + 'static> TileData for &'w mut T {
    type ReadOnly = &'w T;
}

impl<'w, T: Send + Sync + 'static> TileDataQuery for &'w mut T {
    type Item<'a> = &'a mut T;

    type Source = &'static mut ChunkData<T>;

    fn get<'a>(
        source: <<Self as TileDataQuery>::Source as WorldQuery>::Item<'_>,
        index: usize,
    ) -> Option<Self::Item<'_>> {
        source.into_inner().get_mut(index)
    }
}
