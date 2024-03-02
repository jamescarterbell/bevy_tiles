use std::ops::{Deref, DerefMut};

use bevy::{
    ecs::{
        query::{QueryData, QueryFilter, With, WorldQuery},
        system::SystemParam,
    },
    prelude::Query,
};

use crate::{
    chunks::{Chunk, ChunkCoord, InMap},
    maps::{MapLabel, TileMap, TileMapLabel},
    prelude::{
        calculate_chunk_coordinate, calculate_tile_coordinate, calculate_tile_index,
        max_tile_index, CoordIterator,
    },
};

use super::InChunk;

/// Used to query individual tiles from a tile map.
/// This query also implicitly queries chunks and maps
/// in order to properly resolve tiles.
#[derive(SystemParam)]
pub struct TileQuery<'w, 's, L, Q, F = (), const N: usize = 2>
where
    L: TileMapLabel + 'static,
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    tile_q: Query<'w, 's, Q, (F, With<InChunk>, With<MapLabel<L>>)>,
    chunk_q: Query<'w, 's, &'static Chunk, (With<InMap>, With<MapLabel<L>>)>,
    map_q: Query<'w, 's, &'static TileMap<N>, With<MapLabel<L>>>,
}

impl<'w, 's, L, Q, F, const N: usize> Deref for TileQuery<'w, 's, L, Q, F, N>
where
    L: TileMapLabel + 'static,
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    type Target = Query<'w, 's, Q, (F, With<InChunk>, With<MapLabel<L>>)>;

    fn deref(&self) -> &Self::Target {
        &self.tile_q
    }
}

impl<'w, 's, L, Q, F, const N: usize> DerefMut for TileQuery<'w, 's, L, Q, F, N>
where
    L: TileMapLabel + 'static,
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tile_q
    }
}

impl<'w, 's, L, Q, F, const N: usize> TileQuery<'w, 's, L, Q, F, N>
where
    L: TileMapLabel + 'static,
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    /// Get's the readonly query item for the given tile.
    pub fn get_at(
        &self,
        tile_c: [isize; N],
    ) -> Option<<<Q as QueryData>::ReadOnly as WorldQuery>::Item<'_>> {
        let map = self.map_q.get_single().ok()?;
        let chunk_c = calculate_chunk_coordinate(tile_c, L::CHUNK_SIZE);
        let chunk_e = map.chunks.get::<ChunkCoord<N>>(&chunk_c.into())?;

        let chunk = self.chunk_q.get(*chunk_e).ok()?;
        let tile_index = calculate_tile_index(tile_c, L::CHUNK_SIZE);
        let tile_e = chunk.tiles.get(tile_index)?.as_ref()?;

        self.tile_q.get(*tile_e).ok()
    }

    /// Get's the query item for the given tile.
    pub fn get_at_mut(&mut self, tile_c: [isize; N]) -> Option<<Q as WorldQuery>::Item<'_>> {
        let map = self.map_q.get_single().ok()?;
        let chunk_c = calculate_chunk_coordinate(tile_c, L::CHUNK_SIZE);
        let chunk_e = map.chunks.get::<ChunkCoord<N>>(&chunk_c.into())?;

        let chunk = self.chunk_q.get(*chunk_e).ok()?;
        let tile_i = calculate_tile_index(tile_c, L::CHUNK_SIZE);
        let tile_e = chunk.tiles.get(tile_i)?.as_ref()?;

        self.tile_q.get_mut(*tile_e).ok()
    }

    /// Get's the query item for the given tile.
    /// # Safety
    /// This function makes it possible to violate Rust's aliasing guarantees: please use responsibly.
    pub unsafe fn get_at_unchecked(
        &self,
        tile_c: [isize; N],
    ) -> Option<<Q as WorldQuery>::Item<'_>> {
        let map = self.map_q.get_single().ok()?;
        let chunk_c = calculate_chunk_coordinate(tile_c, L::CHUNK_SIZE);
        let chunk_e = map.chunks.get::<ChunkCoord<N>>(&chunk_c.into())?;

        let chunk = self.chunk_q.get(*chunk_e).ok()?;
        let tile_i = calculate_tile_index(tile_c, L::CHUNK_SIZE);
        let tile_e = chunk.tiles.get(tile_i)?.as_ref()?;

        self.tile_q.get_unchecked(*tile_e).ok()
    }

    /// Iterate over all the tiles in a given space, starting at `corner_1`
    /// inclusive over `corner_2`
    pub fn iter_in(
        &self,
        corner_1: [isize; N],
        corner_2: [isize; N],
    ) -> TileQueryIter<'_, 's, L, Q, F, N> {
        TileQueryIter::new(self, corner_1, corner_2)
    }

    /// Iterate over all the tiles in a given space, starting at `corner_1`
    /// inclusive over `corner_2`
    pub fn iter_in_mut(
        &mut self,
        corner_1: [isize; N],
        corner_2: [isize; N],
    ) -> TileQueryIterMut<'_, 's, L, Q, F, N> {
        // SAFETY: Use case is safe since this is the mut version and the function signature
        // stops us from borrowing this mutably twice
        unsafe { TileQueryIterMut::new(self, corner_1, corner_2) }
    }

    /// Get the readonly version of this query.
    pub fn to_readonly(&self) -> TileQuery<'_, 's, L, <Q as QueryData>::ReadOnly, F, N> {
        TileQuery::<L, <Q as QueryData>::ReadOnly, F, N> {
            tile_q: self.tile_q.to_readonly(),
            chunk_q: self.chunk_q.to_readonly(),
            map_q: self.map_q.to_readonly(),
        }
    }

    /// Iter all tiles in a given chunk.
    /// # Note
    /// The coordinates for this function are givne in chunk coordinates.
    pub fn iter_in_chunk(&self, chunk_c: [isize; N]) -> TileQueryIter<'_, 's, L, Q, F, N> {
        // Get corners of chunk
        let corner_1 = calculate_tile_coordinate(chunk_c, 0, L::CHUNK_SIZE);
        let corner_2 =
            calculate_tile_coordinate(chunk_c, max_tile_index::<N>(L::CHUNK_SIZE), L::CHUNK_SIZE);
        // Create tile iter
        TileQueryIter::new(self, corner_1, corner_2)
    }

    /// Iter all tiles in a given chunk.
    /// # Note
    /// The coordinates for this function are givne in chunk coordinates.
    pub fn iter_in_chunk_mut(
        &mut self,
        chunk_c: [isize; N],
    ) -> TileQueryIterMut<'_, 's, L, Q, F, N> {
        // Get corners of chunk
        let corner_1 = calculate_tile_coordinate(chunk_c, 0, L::CHUNK_SIZE);
        let corner_2 =
            calculate_tile_coordinate(chunk_c, max_tile_index::<N>(L::CHUNK_SIZE), L::CHUNK_SIZE);

        // SAFETY: Use case is safe since this is the mut version and the function signature
        // stops us from borrowing this mutably twice
        unsafe { TileQueryIterMut::new(self, corner_1, corner_2) }
    }

    /// Iter all tiles in the chunks in the given range.
    /// # Note
    /// The coordinates for this function are givne in chunk coordinates.
    pub fn iter_in_chunks(
        &mut self,
        chunk_c_1: [isize; N],
        chunk_c_2: [isize; N],
    ) -> TileQueryIter<'_, 's, L, Q, F, N> {
        // Get corners of chunk
        let corner_1 = calculate_tile_coordinate(chunk_c_1, 0, L::CHUNK_SIZE);
        let corner_2 =
            calculate_tile_coordinate(chunk_c_2, max_tile_index::<N>(L::CHUNK_SIZE), L::CHUNK_SIZE);
        // Create tile iter
        TileQueryIter::new(self, corner_1, corner_2)
    }

    /// Iter all tiles in the chunks in the given range.
    /// # Note
    /// The coordinates for this function are givne in chunk coordinates.
    pub fn iter_in_chunks_mut(
        &mut self,
        chunk_c_1: [isize; N],
        chunk_c_2: [isize; N],
    ) -> TileQueryIterMut<'_, 's, L, Q, F, N> {
        // Get corners of chunk
        let corner_1 = calculate_tile_coordinate(chunk_c_1, 0, L::CHUNK_SIZE);
        let corner_2 =
            calculate_tile_coordinate(chunk_c_2, max_tile_index::<N>(L::CHUNK_SIZE), L::CHUNK_SIZE);

        // SAFETY: Use case is safe since this is the mut version and the function signature
        // stops us from borrowing this mutably twice
        unsafe { TileQueryIterMut::new(self, corner_1, corner_2) }
    }
}

/// Iterates over all the tiles in a region.
pub struct TileQueryIter<'w, 's, L, Q, F, const N: usize>
where
    L: TileMapLabel + 'static,
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    coord_iter: CoordIterator<N>,
    tile_q: &'w TileQuery<'w, 's, L, Q, F, N>,
}

impl<'w, 's, L, Q, F, const N: usize> TileQueryIter<'w, 's, L, Q, F, N>
where
    L: TileMapLabel + 'static,
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    /// # Safety
    /// This iterator uses unchecked get's to get around some lifetime issue I don't understand yet.
    /// Due to this, you should only call this constructor from a context where the query is actually
    /// borrowed mutabley.
    fn new(
        tile_q: &'w TileQuery<'w, 's, L, Q, F, N>,
        corner_1: [isize; N],
        corner_2: [isize; N],
    ) -> Self {
        Self {
            tile_q,
            coord_iter: CoordIterator::new(corner_1, corner_2),
        }
    }
}

impl<'w, 's, L, Q, F, const N: usize> Iterator for TileQueryIter<'w, 's, L, Q, F, N>
where
    L: TileMapLabel + 'static,
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    type Item = <<Q as QueryData>::ReadOnly as WorldQuery>::Item<'w>;

    #[allow(clippy::while_let_on_iterator)]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(target) = self.coord_iter.next() {
            // This fixes some lifetime issue that I'm not sure I understand quite yet, will do testing
            let tile = self.tile_q.get_at(target);
            if tile.is_some() {
                return tile;
            }
        }

        None
    }
}

/// ```compile_fail
///# // Because we're using unsafe, we need to make sure we don't mutabley alias.
///# fn multiple_iter_mut(mut tile_query: TileQuery<TestLayer, ()>) {
///#     let mut iter_1 = tile_query.iter_in([0, 0], [3, 3]);
///#     let mut iter_2 = tile_query.iter_in_mut([0, 0], [3, 3]);
///#     let _ = iter_1.next();
///#     let _ = iter_2.next();
///# }
/// ```
pub struct TileQueryIterMut<'w, 's, L, Q, F, const N: usize>
where
    L: TileMapLabel + 'static,
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    coord_iter: CoordIterator<N>,
    tile_q: &'w TileQuery<'w, 's, L, Q, F, N>,
}

impl<'w, 's, L, Q, F, const N: usize> TileQueryIterMut<'w, 's, L, Q, F, N>
where
    L: TileMapLabel + 'static,
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    /// # Safety
    /// This iterator uses unchecked get's to get around some lifetime issue I don't understand yet.
    /// Due to this, you should only call this constructor from a context where the query is actually
    /// borrowed mutabley.
    unsafe fn new(
        tile_q: &'w TileQuery<'w, 's, L, Q, F, N>,
        corner_1: [isize; N],
        corner_2: [isize; N],
    ) -> Self {
        Self {
            tile_q,
            coord_iter: CoordIterator::new(corner_1, corner_2),
        }
    }
}

impl<'w, 's, L, Q, F, const N: usize> Iterator for TileQueryIterMut<'w, 's, L, Q, F, N>
where
    L: TileMapLabel + 'static,
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    type Item = <Q as WorldQuery>::Item<'w>;

    #[allow(clippy::while_let_on_iterator)]
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
