use std::any::TypeId;

use bevy::{
    ecs::query::{QueryData, WorldQuery},
    prelude::{Bundle, Component, Entity, EntityWorldMut},
};

use crate::chunks::{ChunkData, ChunkTypes};

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

impl TileData for Entity {
    type ReadOnly = Self;
}

/// Safety: Entity is readonly.
unsafe impl ReadOnlyTileData for Entity {}

impl TileDataQuery for Entity {
    type Item<'a> = Entity;

    type Source = &'static ChunkData<Entity>;

    fn get<'a>(
        source: <<Self as TileDataQuery>::Source as WorldQuery>::Item<'_>,
        index: usize,
    ) -> Option<Self::Item<'_>> {
        source.get(index).cloned()
    }
}

/// The tiled version of a component bundle.
/// # Safety
/// Easy to screw this up.
pub unsafe trait TileBundle: Sized + Send + Sync + 'static {
    /// The type returned when a value is replaced (Usually an Option or group of Options).
    type Replaced: Default;
    /// The component on the chunk tile data is queried from.
    type Source: Component;

    /// Inserts a bundle and returns all the replaced values.
    fn insert_tile_into_chunk<const N: usize>(
        self,
        chunk: EntityWorldMut<'_>,
        chunk_size: usize,
        tile_i: usize,
    ) -> Self::Replaced;

    /// Try to remove a bundle.
    fn take_tile_from_chunk(chunk: &mut EntityWorldMut<'_>, tile_i: usize) -> Self::Replaced;
}

/// # Safety:
/// Probably safe.
unsafe impl<T: Sized + Send + Sync + 'static> TileBundle for T {
    type Replaced = Option<T>;

    type Source = ChunkData<T>;

    fn insert_tile_into_chunk<const N: usize>(
        self,
        mut chunk: EntityWorldMut<'_>,
        chunk_size: usize,
        tile_i: usize,
    ) -> Self::Replaced {
        let location = match chunk.get_mut::<Self::Source>() {
            Some(data) => data,
            None => {
                chunk
                    .get_mut::<ChunkTypes>()
                    .unwrap()
                    .0
                    .insert(TypeId::of::<Self>());
                let chunk = chunk.insert(Self::Source::new(chunk_size.pow(N.try_into().unwrap())));
                chunk.get_mut::<Self::Source>().unwrap()
            }
        };
        let mut binding = location;
        binding.insert(tile_i, self)
    }

    fn take_tile_from_chunk(chunk: &mut EntityWorldMut<'_>, tile_i: usize) -> Self::Replaced {
        let location = chunk.get_mut::<Self::Source>();
        let mut binding = location?;
        let removed = binding.take(tile_i);
        if binding.count == 0 {
            chunk
                .get_mut::<ChunkTypes>()
                .unwrap()
                .0
                .remove(&TypeId::of::<Self>());
            chunk.remove::<Self::Source>();
        }
        removed
    }
}
