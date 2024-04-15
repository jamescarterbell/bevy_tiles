use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use bevy::{
    ecs::{
        entity::Entity,
        query::{QueryData, QueryFilter, With, WorldQuery},
        system::SystemParam,
    },
    prelude::Query,
};

use crate::{
    chunks::{Chunk, InMap},
    maps::TileMap,
    prelude::{calculate_tile_coordinate, calculate_tile_index, max_tile_index, CoordIterator},
};

use super::{InChunk, TileCoord};

/// Borrowed types from a [TileMapQuery] needed to construct a [TileQuery]
pub struct BorrowedTileQueries<'a, 'w: 'a, 's: 'a, Q, F, const N: usize>
where
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    phantom: PhantomData<(&'a Q, &'w F, &'s Q)>,
}

impl<'a, 'w: 'a, 's: 'a, Q, F, const N: usize> BorrowedTileQueryTypes<'a>
    for BorrowedTileQueries<'a, 'w, 's, Q, F, N>
where
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    type TileQuery = &'a Query<'w, 's, Q, (F, With<InChunk>)>;

    type ChunkQuery = &'a Query<'w, 's, &'static Chunk, With<InMap>>;

    type Map = &'a TileMap<N>;
}

/// Mutable borrowed types from a [TileMapQuery] needed to construct a mutable [TileQuery]
pub struct MutableBorrowedTileQueries<'a, 'w: 'a, 's: 'a, Q, F, const N: usize>
where
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    phantom: PhantomData<(&'a Q, &'w F, &'s Q)>,
}

impl<'a, 'w: 'a, 's: 'a, Q, F, const N: usize> BorrowedTileQueryTypes<'a>
    for MutableBorrowedTileQueries<'a, 'w, 's, Q, F, N>
where
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    type TileQuery = &'a mut Query<'w, 's, Q, (F, With<InChunk>)>;

    type ChunkQuery = &'a mut Query<'w, 's, &'static Chunk, With<InMap>>;

    type Map = &'a TileMap<N>;
}

/// Describes the types used to construct a query, mainly needed to reduce code duplication.
pub trait BorrowedTileQueryTypes<'a> {
    /// Query for tiles.
    type TileQuery;
    /// Query for chunks.
    type ChunkQuery;
    /// The map used.
    type Map;
}

/// Used to query individual tiles from a tile map.
/// This query also implicitly queries chunks and maps
/// in order to properly resolve tiles.
#[derive(SystemParam)]
pub struct TileMapQuery<'w, 's, Q, F = (), const N: usize = 2>
where
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    tile_q: Query<'w, 's, Q, (F, With<InChunk>)>,
    chunk_q: Query<'w, 's, &'static Chunk, With<InMap>>,
    map_q: Query<'w, 's, &'static TileMap<N>>,
}

impl<'w, 's, Q, F, const N: usize> TileMapQuery<'w, 's, Q, F, N>
where
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    /// Gets the query for a given map.
    pub fn get_map(
        &self,
        map_id: Entity,
    ) -> Option<
        TileQuery<
            &'_ Query<'w, 's, Q, (F, With<InChunk>)>,
            &'_ Query<'w, 's, &'static Chunk, With<InMap>>,
            N,
        >,
    > {
        let map = self.map_q.get(map_id).ok()?;

        Some(TileQuery {
            tile_q: &self.tile_q,
            chunk_q: &self.chunk_q,
            map,
        })
    }

    /// Gets the query for a given map.
    pub fn get_map_mut(
        &mut self,
        map_id: Entity,
    ) -> Option<
        TileQuery<
            &'_ mut Query<'w, 's, Q, (F, With<InChunk>)>,
            &'_ mut Query<'w, 's, &'static Chunk, With<InMap>>,
            N,
        >,
    > {
        let map = self.map_q.get(map_id).ok()?;

        Some(TileQuery {
            tile_q: &mut self.tile_q,
            chunk_q: &mut self.chunk_q,
            map,
        })
    }
}

/// Queries a particular tilemap.
pub struct TileQuery<'a, T, C, const N: usize> {
    tile_q: T,
    chunk_q: C,
    map: &'a TileMap<N>,
}

impl<'a, 'w: 'a, 's: 'a, Q, F, T, C, const N: usize> TileQuery<'a, T, C, N>
where
    T: Deref<Target = Query<'w, 's, Q, (F, With<InChunk>)>>,
    C: Deref<Target = Query<'w, 's, &'static Chunk, With<InMap>>>,
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    fn get_tile_entity(&self, tile_c: [isize; N]) -> Option<Entity> {
        let chunk_size = self.map.get_chunk_size();
        let chunk_id = self.map.get_from_tile(TileCoord::<N>(tile_c))?;

        let chunk = self.chunk_q.get(chunk_id).ok()?;
        let tile_index = calculate_tile_index(tile_c, chunk_size);

        chunk.get(tile_index)
    }

    /// Gets the readonly query item for the given tile.
    pub fn get_at(
        &self,
        tile_c: [isize; N],
    ) -> Option<<<Q as QueryData>::ReadOnly as WorldQuery>::Item<'_>> {
        let tile_e = self.get_tile_entity(tile_c)?;
        self.tile_q.get(tile_e).ok()
    }

    /// Gets the query item for the given tile.
    /// # Safety
    /// This function makes it possible to violate Rust's aliasing guarantees: please use responsibly.
    pub unsafe fn get_at_unchecked(
        &self,
        tile_c: [isize; N],
    ) -> Option<<Q as WorldQuery>::Item<'_>> {
        let tile_e = self.get_tile_entity(tile_c)?;
        self.tile_q.get_unchecked(tile_e).ok()
    }

    /// Iterate over all the tiles in a given space, starting at `corner_1`
    /// inclusive over `corner_2`
    pub fn iter_in(
        &self,
        corner_1: [isize; N],
        corner_2: [isize; N],
    ) -> TileQueryIter<'_, 'a, T, C, N> {
        TileQueryIter::new(self, corner_1, corner_2)
    }

    /// Iter all tiles in a given chunk.
    /// # Note
    /// The coordinates for this function are givne in chunk coordinates.
    pub fn iter_in_chunk(&self, chunk_c: [isize; N]) -> TileQueryIter<'_, 'a, T, C, N> {
        let chunk_size = self.map.get_chunk_size();
        // Get corners of chunk
        let corner_1 = calculate_tile_coordinate(chunk_c, 0, chunk_size);
        let corner_2 =
            calculate_tile_coordinate(chunk_c, max_tile_index::<N>(chunk_size), chunk_size);
        // Create tile iter
        TileQueryIter::new(self, corner_1, corner_2)
    }

    /// Iter all tiles in the chunks in the given range.
    /// # Note
    /// The coordinates for this function are givne in chunk coordinates.
    pub fn iter_in_chunks(
        &mut self,
        chunk_c_1: [isize; N],
        chunk_c_2: [isize; N],
    ) -> TileQueryIter<'_, 'a, T, C, N> {
        let chunk_size = self.map.get_chunk_size();
        // Get corners of chunk
        let corner_1 = calculate_tile_coordinate(chunk_c_1, 0, chunk_size);
        let corner_2 =
            calculate_tile_coordinate(chunk_c_2, max_tile_index::<N>(chunk_size), chunk_size);
        // Create tile iter
        TileQueryIter::new(self, corner_1, corner_2)
    }
}

impl<'a, 'w: 'a, 's: 'a, Q, F, T, C, const N: usize> TileQuery<'a, T, C, N>
where
    T: DerefMut<Target = Query<'w, 's, Q, (F, With<InChunk>)>>,
    C: DerefMut<Target = Query<'w, 's, &'static Chunk, With<InMap>>>,
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    /// Gets the query item for the given tile.
    pub fn get_at_mut(&mut self, tile_c: [isize; N]) -> Option<<Q as WorldQuery>::Item<'_>> {
        let tile_e = self.get_tile_entity(tile_c)?;
        self.tile_q.get_mut(tile_e).ok()
    }

    /// Iter all tiles in the chunks in the given range.
    /// # Note
    /// The coordinates for this function are givne in chunk coordinates.
    pub fn iter_in_chunks_mut(
        &mut self,
        chunk_c_1: [isize; N],
        chunk_c_2: [isize; N],
    ) -> TileQueryIterMut<'_, 'a, T, C, N> {
        let chunk_size = self.map.get_chunk_size();
        // Get corners of chunk
        let corner_1 = calculate_tile_coordinate(chunk_c_1, 0, chunk_size);
        let corner_2 =
            calculate_tile_coordinate(chunk_c_2, max_tile_index::<N>(chunk_size), chunk_size);

        TileQueryIterMut::new(self, corner_1, corner_2)
    }

    /// Iter all tiles in a given chunk.
    /// # Note
    /// The coordinates for this function are givne in chunk coordinates.
    pub fn iter_in_chunk_mut(&mut self, chunk_c: [isize; N]) -> TileQueryIterMut<'_, 'a, T, C, N> {
        let chunk_size = self.map.get_chunk_size();
        // Get corners of chunk
        let corner_1 = calculate_tile_coordinate(chunk_c, 0, chunk_size);
        let corner_2 =
            calculate_tile_coordinate(chunk_c, max_tile_index::<N>(chunk_size), chunk_size);

        TileQueryIterMut::new(self, corner_1, corner_2)
    }

    /// Iterate over all the tiles in a given space, starting at `corner_1`
    /// inclusive over `corner_2`
    pub fn iter_in_mut(
        &mut self,
        corner_1: [isize; N],
        corner_2: [isize; N],
    ) -> TileQueryIterMut<'_, 'a, T, C, N> {
        TileQueryIterMut::new(self, corner_1, corner_2)
    }
}

/// Iterates over all the tiles in a region.
pub struct TileQueryIter<'i, 'a, T, C, const N: usize> {
    coord_iter: CoordIterator<N>,
    tile_q: &'i TileQuery<'a, T, C, N>,
}

impl<'i, 'a: 'i, T, C, const N: usize> TileQueryIter<'i, 'a, T, C, N> {
    fn new(tile_q: &'i TileQuery<'a, T, C, N>, corner_1: [isize; N], corner_2: [isize; N]) -> Self {
        Self {
            tile_q,
            coord_iter: CoordIterator::new(corner_1, corner_2),
        }
    }
}

impl<'i, 'a: 'i, 'w: 'a, 's: 'a, Q, F, T, C, const N: usize> Iterator
    for TileQueryIter<'i, 'a, T, C, N>
where
    T: Deref<Target = Query<'w, 's, Q, (F, With<InChunk>)>>,
    C: Deref<Target = Query<'w, 's, &'static Chunk, With<InMap>>>,
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    type Item = <<Q as QueryData>::ReadOnly as WorldQuery>::Item<'i>;

    #[allow(clippy::while_let_on_iterator)]
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

/// ```compile_fail
///# // Because we're using unsafe, we need to make sure we don't mutabley alias.
///# fn multiple_iter_mut(mut tile_query: TileQuery<TestLayer, ()>) {
///#     let mut iter_1 = tile_query.iter_in([0, 0], [3, 3]);
///#     let mut iter_2 = tile_query.iter_in_mut([0, 0], [3, 3]);
///#     let _ = iter_1.next();
///#     let _ = iter_2.next();
///# }
/// ```
/// Iterates over all the tiles in a region.
pub struct TileQueryIterMut<'i, 'a, T, C, const N: usize> {
    coord_iter: CoordIterator<N>,
    tile_q: &'i TileQuery<'a, T, C, N>,
}

impl<'i, 'a: 'i, T, C, const N: usize> TileQueryIterMut<'i, 'a, T, C, N> {
    fn new(
        tile_q: &'i mut TileQuery<'a, T, C, N>,
        corner_1: [isize; N],
        corner_2: [isize; N],
    ) -> Self {
        Self {
            tile_q,
            coord_iter: CoordIterator::new(corner_1, corner_2),
        }
    }
}

impl<'i, 'a: 'i, 'w: 'a, 's: 'a, Q, F, T, C, const N: usize> Iterator
    for TileQueryIterMut<'i, 'a, T, C, N>
where
    T: DerefMut<Target = Query<'w, 's, Q, (F, With<InChunk>)>>,
    C: Deref<Target = Query<'w, 's, &'static Chunk, With<InMap>>>,
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    type Item = <Q as WorldQuery>::Item<'i>;

    #[allow(clippy::while_let_on_iterator)]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(target) = self.coord_iter.next() {
            // SAFETY: This is safe as long as new always requires a mutable reference
            let tile = unsafe { self.tile_q.get_at_unchecked(target) };
            if tile.is_some() {
                return tile;
            }
        }

        None
    }
}
