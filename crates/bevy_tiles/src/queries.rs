use std::any::TypeId;

use bevy::{
    ecs::query::{QueryData, WorldQuery},
    prelude::{Bundle, Component, Entity, EntityWorldMut},
};

use crate::{
    chunks::{ChunkData, ChunkTypes},
    maps::{TileDims, TileSpacing},
};

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

/// The tiled version of a component bundle.
/// # Safety
/// Easy to screw this up.
pub unsafe trait TileComponent: Sized + Send + Sync + 'static {
    /// Inserts a bundle and returns all the replaced values.
    fn insert_tile_into_chunk<const N: usize>(
        self,
        chunk: EntityWorldMut<'_>,
        chunk_c: [i32; N],
        chunk_size: usize,
        use_transforms: bool,
        tile_dims: Option<TileDims<N>>,
        tile_spacing: Option<TileSpacing<N>>,
        tile_c: [i32; N],
        tile_i: usize,
    ) -> Option<Self>;

    /// Inserts a bundle and returns all the replaced values.
    fn insert_tile_batch_into_chunk<const N: usize>(
        tiles: impl Iterator<Item = Self>,
        chunk: EntityWorldMut<'_>,
        chunk_c: [i32; N],
        chunk_size: usize,
        use_transforms: bool,
        tile_dims: Option<TileDims<N>>,
        tile_spacing: Option<TileSpacing<N>>,
        tile_is: impl Iterator<Item = ([i32; N], usize)>,
    ) -> impl Iterator<Item = Self>;

    /// Try to remove a bundle.
    fn take_tile_from_chunk(chunk: &mut EntityWorldMut<'_>, tile_i: usize) -> Option<Self>;
}

// /// # Safety:
// /// Probably safe.
// /// MAKE THIS NOT A DEFAULT IMPL
// unsafe impl<T: Sized + Send + Sync + 'static> TileComponent for T {
//     fn insert_tile_into_chunk<const N: usize>(
//         self,
//         mut chunk: EntityWorldMut<'_>,
//         chunk_size: usize,
//         tile_i: usize,
//     ) -> Option<T> {
//         let location = match chunk.get_mut::<ChunkData<Self>>() {
//             Some(data) => data,
//             None => {
//                 chunk
//                     .get_mut::<ChunkTypes>()
//                     .unwrap()
//                     .0
//                     .insert(TypeId::of::<Self>());
//                 let chunk = chunk.insert(ChunkData::<Self>::new(
//                     chunk_size.pow(N.try_into().unwrap()),
//                 ));
//                 chunk.get_mut::<ChunkData<Self>>().unwrap()
//             }
//         };
//         let mut binding = location;
//         binding.insert(tile_i, self)
//     }

//     fn take_tile_from_chunk(chunk: &mut EntityWorldMut<'_>, tile_i: usize) -> Option<Self> {
//         let location = chunk.get_mut::<ChunkData<Self>>();
//         let mut binding = location?;
//         let removed = binding.take(tile_i);
//         if binding.count == 0 {
//             chunk
//                 .get_mut::<ChunkTypes>()
//                 .unwrap()
//                 .0
//                 .remove(&TypeId::of::<Self>());
//             chunk.remove::<ChunkData<Self>>();
//         }
//         removed
//     }

//     fn insert_tile_batch_into_chunk<const N: usize>(
//         tiles: impl Iterator<Item = Self>,
//         mut chunk: EntityWorldMut<'_>,
//         chunk_size: usize,
//         tile_is: impl Iterator<Item = usize>,
//     ) {
//         let location = match chunk.get_mut::<ChunkData<Self>>() {
//             Some(data) => data,
//             None => {
//                 chunk
//                     .get_mut::<ChunkTypes>()
//                     .unwrap()
//                     .0
//                     .insert(TypeId::of::<Self>());
//                 let chunk = chunk.insert(ChunkData::<Self>::new(
//                     chunk_size.pow(N.try_into().unwrap()),
//                 ));
//                 chunk.get_mut::<ChunkData<Self>>().unwrap()
//             }
//         };
//         let mut binding = location;
//         for (tile_i, tile) in tile_is.zip(tiles) {
//             binding.insert(tile_i, tile);
//         }
//     }
// }
