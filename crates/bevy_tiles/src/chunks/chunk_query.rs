use std::ops::{Deref, DerefMut};

use aery::prelude::*;
use bevy::{
    ecs::{
        prelude::With,
        query::{ReadOnlyWorldQuery, WorldQuery},
        system::SystemParam,
    },
    prelude::Query,
};

use crate::{
    chunks::ChunkCoord,
    maps::{MapLabel, TileMap, TileMapLabel},
    prelude::{calculate_chunk_coordinate, Chunk, CoordIterator, InMap},
};

/// Used to query chunks from a tile map.
/// This query also implicitly queries maps
/// in order to properly resolve chunks.
#[derive(SystemParam)]
pub struct ChunkQuery<'w, 's, L, Q, F = (), const N: usize = 2>
where
    L: TileMapLabel + 'static,
    Q: WorldQuery + 'static,
    F: ReadOnlyWorldQuery + 'static,
{
    chunk_q: Query<'w, 's, Q, (F, Relations<InMap<L, N>>, With<Chunk>)>,
    map_q: Query<'w, 's, &'static TileMap<N>, With<MapLabel<L>>>,
}

impl<'w, 's, L, Q, F, const N: usize> Deref for ChunkQuery<'w, 's, L, Q, F, N>
where
    L: TileMapLabel + 'static,
    Q: WorldQuery + 'static,
    F: ReadOnlyWorldQuery + 'static,
{
    type Target = Query<'w, 's, Q, (F, Relations<InMap<L, N>>, With<Chunk>)>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.chunk_q
    }
}

impl<'w, 's, L, Q, F, const N: usize> DerefMut for ChunkQuery<'w, 's, L, Q, F, N>
where
    L: TileMapLabel + 'static,
    Q: WorldQuery + 'static,
    F: ReadOnlyWorldQuery + 'static,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.chunk_q
    }
}

impl<'w, 's, L, Q, F, const N: usize> ChunkQuery<'w, 's, L, Q, F, N>
where
    L: TileMapLabel + 'static,
    Q: WorldQuery + 'static,
    F: ReadOnlyWorldQuery + 'static,
{
    /// Get's the readonly query item for the given tile.
    /// # Note
    /// Coordinates are for these calls are in chunk coordinates.
    #[inline]
    pub fn get_at(
        &self,
        tile_c: [isize; N],
    ) -> Option<<<Q as WorldQuery>::ReadOnly as WorldQuery>::Item<'_>> {
        let map = self.map_q.get_single().ok()?;
        let chunk_c = calculate_chunk_coordinate(tile_c, L::CHUNK_SIZE);
        let chunk_e = map.chunks.get::<ChunkCoord<N>>(&chunk_c.into())?;

        self.chunk_q.get(*chunk_e).ok()
    }

    /// Get's the query item for the given tile.
    /// # Note
    /// Coordinates are for these calls are in chunk coordinates.
    #[inline]
    pub fn get_at_mut(&mut self, tile_c: [isize; N]) -> Option<<Q as WorldQuery>::Item<'_>> {
        let map = self.map_q.get_single().ok()?;
        let chunk_c = calculate_chunk_coordinate(tile_c, L::CHUNK_SIZE);
        let chunk_e = map.chunks.get::<ChunkCoord<N>>(&chunk_c.into())?;

        self.chunk_q.get_mut(*chunk_e).ok()
    }

    /// Get's the query item for the given chunk.
    /// # Safety
    /// This function makes it possible to violate Rust's aliasing guarantees: please use responsibly.
    /// # Note
    /// Coordinates are for these calls are in chunk coordinates.
    #[inline]
    pub unsafe fn get_at_unchecked(
        &self,
        tile_c: [isize; N],
    ) -> Option<<Q as WorldQuery>::Item<'_>> {
        let map = self.map_q.get_single().ok()?;
        let chunk_c = calculate_chunk_coordinate(tile_c, L::CHUNK_SIZE);
        let chunk_e = map.chunks.get::<ChunkCoord<N>>(&chunk_c.into())?;

        self.chunk_q.get_unchecked(*chunk_e).ok()
    }

    /// Iterate over all the chunks in a given space, starting at `corner_1`
    /// inclusive over `corner_2`
    /// # Note
    /// Coordinates are for these calls are in chunk coordinates.
    #[inline]
    pub fn iter_in(
        &self,
        corner_1: [isize; N],
        corner_2: [isize; N],
    ) -> ChunkQueryIter<'_, 's, L, Q, F, N> {
        ChunkQueryIter::new(self, corner_1, corner_2)
    }

    /// Iterate over all the chunks in a given space, starting at `corner_1`
    /// inclusive over `corner_2`.
    /// # Note
    /// Coordinates are for these calls are in chunk coordinates.
    #[inline]
    pub fn iter_in_mut(
        &mut self,
        corner_1: [isize; N],
        corner_2: [isize; N],
    ) -> ChunkQueryIterMut<'_, 's, L, Q, F, N> {
        // SAFETY: Use case is safe since this is the mut version and the function signature
        // stops us from borrowing this mutably twice
        unsafe { ChunkQueryIterMut::new(self, corner_1, corner_2) }
    }

    /// Get the readonly version of this query.
    #[inline]
    pub fn to_readonly(
        &self,
    ) -> ChunkQuery<'_, 's, L, <Q as WorldQuery>::ReadOnly, <F as WorldQuery>::ReadOnly, N> {
        ChunkQuery::<L, <Q as WorldQuery>::ReadOnly, <F as WorldQuery>::ReadOnly, N> {
            chunk_q: self.chunk_q.to_readonly(),
            map_q: self.map_q.to_readonly(),
        }
    }
}

/// Iterates over a range of chunks using chunk coordinates.
pub struct ChunkQueryIter<'w, 's, L, Q, F, const N: usize>
where
    L: TileMapLabel + 'static,
    Q: WorldQuery + 'static,
    F: ReadOnlyWorldQuery + 'static,
{
    coord_iter: CoordIterator<N>,
    tile_q: &'w ChunkQuery<'w, 's, L, Q, F, N>,
}

impl<'w, 's, L, Q, F, const N: usize> ChunkQueryIter<'w, 's, L, Q, F, N>
where
    L: TileMapLabel + 'static,
    Q: WorldQuery + 'static,
    F: ReadOnlyWorldQuery + 'static,
{
    /// # Safety
    /// This iterator uses unchecked get's to get around some lifetime issue I don't understand yet.
    /// Due to this, you should only call this constructor from a context where the query is actually
    /// borrowed mutabley.
    fn new(
        tile_q: &'w ChunkQuery<'w, 's, L, Q, F, N>,
        corner_1: [isize; N],
        corner_2: [isize; N],
    ) -> Self {
        Self {
            tile_q,
            coord_iter: CoordIterator::new(corner_1, corner_2),
        }
    }
}

impl<'w, 's, L, Q, F, const N: usize> Iterator for ChunkQueryIter<'w, 's, L, Q, F, N>
where
    L: TileMapLabel + 'static,
    Q: WorldQuery + 'static,
    F: ReadOnlyWorldQuery + 'static,
{
    type Item = <<Q as WorldQuery>::ReadOnly as WorldQuery>::Item<'w>;

    #[allow(clippy::while_let_on_iterator)]
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(target) = self.coord_iter.next() {
            let tile = self.tile_q.get_at(target);
            if tile.is_some() {
                return tile;
            }
        }

        None
    }
}

/// Iterates over a range of chunks mutably using chunk coordinates.
/// # Note
/// Due to weird borrow checker stuff, this is a seperate struct.
/// In the future, we may find a way to combine the iterators.
/// ```compile_fail
///# // Because we're using unsafe, we need to make sure we don't mutabley alias.
///# fn multiple_iter_mut(mut tile_query: ChunkQuery<TestLayer, ()>) {
///#     let mut iter_1 = tile_query.iter_in([0, 0], [3, 3]);
///#     let mut iter_2 = tile_query.iter_in_mut([0, 0], [3, 3]);
///#     let _ = iter_1.next();
///#     let _ = iter_2.next();
///# }
/// ```
pub struct ChunkQueryIterMut<'w, 's, L, Q, F, const N: usize>
where
    L: TileMapLabel + 'static,
    Q: WorldQuery + 'static,
    F: ReadOnlyWorldQuery + 'static,
{
    coord_iter: CoordIterator<N>,
    tile_q: &'w ChunkQuery<'w, 's, L, Q, F, N>,
}

impl<'w, 's, L, Q, F, const N: usize> ChunkQueryIterMut<'w, 's, L, Q, F, N>
where
    L: TileMapLabel + 'static,
    Q: WorldQuery + 'static,
    F: ReadOnlyWorldQuery + 'static,
{
    /// # Safety
    /// This iterator uses unchecked get's to get around some lifetime issue I don't understand yet.
    /// Due to this, you should only call this constructor from a context where the query is actually
    /// borrowed mutabley.
    unsafe fn new(
        tile_q: &'w ChunkQuery<'w, 's, L, Q, F, N>,
        corner_1: [isize; N],
        corner_2: [isize; N],
    ) -> Self {
        Self {
            tile_q,
            coord_iter: CoordIterator::new(corner_1, corner_2),
        }
    }
}

impl<'w, 's, L, Q, F, const N: usize> Iterator for ChunkQueryIterMut<'w, 's, L, Q, F, N>
where
    L: TileMapLabel + 'static,
    Q: WorldQuery + 'static,
    F: ReadOnlyWorldQuery + 'static,
{
    type Item = <Q as WorldQuery>::Item<'w>;

    #[allow(clippy::while_let_on_iterator)]
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(target) = self.coord_iter.next() {
            // SAFETY: This fixes some lifetime issue that I'm not sure I understand quite yet, will do testing
            let tile = unsafe { self.tile_q.get_at_unchecked(target) };
            if tile.is_some() {
                return tile;
            }
        }

        None
    }
}
