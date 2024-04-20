use std::ops::{Deref, DerefMut};

use bevy::{
    ecs::{
        entity::Entity,
        prelude::With,
        query::{QueryData, QueryFilter, WorldQuery},
        system::SystemParam,
    },
    prelude::Query,
};

use crate::{
    chunks::{Chunk, ChunkCoord, InMap},
    coords::CoordIterator,
    maps::TileMap,
};

/// Used to query chunks from any tile map.
/// This query also implicitly queries maps
/// in order to properly resolve chunks.
#[derive(SystemParam)]
pub struct ChunkMapQuery<'w, 's, Q, F = (), const N: usize = 2>
where
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    chunk_q: Query<'w, 's, Q, (F, With<InMap>, With<Chunk>)>,
    map_q: Query<'w, 's, &'static TileMap<N>>,
}

impl<'w, 's, Q, F, const N: usize> ChunkMapQuery<'w, 's, Q, F, N>
where
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    /// Gets the query for a given map.
    pub fn get_map(
        &self,
        map_id: Entity,
    ) -> Option<ChunkQuery<&'_ Query<'w, 's, Q, (F, With<InMap>, With<Chunk>)>, N>> {
        let map = self.map_q.get(map_id).ok()?;

        Some(ChunkQuery {
            chunk_q: &self.chunk_q,
            map,
        })
    }

    /// Gets the query for a given map.
    pub fn get_map_mut(
        &mut self,
        map_id: Entity,
    ) -> Option<ChunkQuery<&'_ mut Query<'w, 's, Q, (F, With<InMap>, With<Chunk>)>, N>> {
        let map = self.map_q.get(map_id).ok()?;

        Some(ChunkQuery {
            chunk_q: &mut self.chunk_q,
            map,
        })
    }
}

/// Used to query chunks from a tile map.
/// This query also implicitly queries maps
/// in order to properly resolve chunks.
pub struct ChunkQuery<'a, C, const N: usize> {
    chunk_q: C,
    map: &'a TileMap<N>,
}

impl<'a, 'w: 'a, 's: 'a, Q, F, C, const N: usize> ChunkQuery<'a, C, N>
where
    C: Deref<Target = Query<'w, 's, Q, (F, With<InMap>, With<Chunk>)>>,
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    /// Get's the readonly query item for the given tile.
    /// # Note
    /// Coordinates are for these calls are in chunk coordinates.
    #[inline]
    pub fn get_at(
        &self,
        chunk_c: impl Into<[i32; N]>,
    ) -> Option<<<Q as QueryData>::ReadOnly as WorldQuery>::Item<'_>> {
        let chunk_c = chunk_c.into();
        let chunk_id = self.map.get_from_chunk(ChunkCoord(chunk_c))?;

        self.chunk_q.get(chunk_id).ok()
    }

    /// Get's the query item for the given chunk.
    /// # Safety
    /// This function makes it possible to violate Rust's aliasing guarantees: please use responsibly.
    /// # Note
    /// Coordinates are for these calls are in chunk coordinates.
    #[inline]
    pub unsafe fn get_at_unchecked(
        &self,
        chunk_c: impl Into<[i32; N]>,
    ) -> Option<<Q as WorldQuery>::Item<'_>> {
        let chunk_c = chunk_c.into();
        let chunk_id = self.map.get_from_chunk(ChunkCoord(chunk_c))?;

        self.chunk_q.get_unchecked(chunk_id).ok()
    }

    /// Iterate over all the chunks in a given space, starting at `corner_1`
    /// inclusive over `corner_2`
    /// # Note
    /// Coordinates are for these calls are in chunk coordinates.
    #[inline]
    pub fn iter_in(
        &self,
        corner_1: impl Into<[i32; N]>,
        corner_2: impl Into<[i32; N]>,
    ) -> ChunkQueryIter<'_, 'a, C, N> {
        let corner_1 = corner_1.into();
        let corner_2 = corner_2.into();
        ChunkQueryIter::new(self, corner_1, corner_2)
    }
}

impl<'a, 'w: 'a, 's: 'a, Q, F, C, const N: usize> ChunkQuery<'a, C, N>
where
    C: DerefMut<Target = Query<'w, 's, Q, (F, With<InMap>, With<Chunk>)>>,
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    /// Get's the query item for the given tile.
    /// # Note
    /// Coordinates are for these calls are in chunk coordinates.
    #[inline]
    pub fn get_at_mut(
        &mut self,
        chunk_c: impl Into<[i32; N]>,
    ) -> Option<<Q as WorldQuery>::Item<'_>> {
        let chunk_c = chunk_c.into();
        let chunk_id = self.map.get_from_chunk(ChunkCoord(chunk_c))?;

        self.chunk_q.get_mut(chunk_id).ok()
    }

    /// Iterate over all the chunks in a given space, starting at `corner_1`
    /// inclusive over `corner_2`.
    /// # Note
    /// Coordinates are for these calls are in chunk coordinates.
    #[inline]
    pub fn iter_in_mut(
        &mut self,
        corner_1: impl Into<[i32; N]>,
        corner_2: impl Into<[i32; N]>,
    ) -> ChunkQueryIterMut<'_, 'a, C, N> {
        let corner_1 = corner_1.into();
        let corner_2 = corner_2.into();
        ChunkQueryIterMut::new(self, corner_1, corner_2)
    }
}

/// Iterates over a range of chunks using chunk coordinates.
pub struct ChunkQueryIter<'i, 'a, C, const N: usize> {
    coord_iter: CoordIterator<N>,
    chunk_q: &'i ChunkQuery<'a, C, N>,
}

impl<'i, 'a, C, const N: usize> ChunkQueryIter<'i, 'a, C, N> {
    fn new(chunk_q: &'i ChunkQuery<'a, C, N>, corner_1: [i32; N], corner_2: [i32; N]) -> Self {
        Self {
            chunk_q,
            coord_iter: CoordIterator::new(corner_1, corner_2),
        }
    }
}

impl<'i, 'a: 'i, 'w: 'a, 's: 'a, Q, F, C, const N: usize> Iterator for ChunkQueryIter<'i, 'a, C, N>
where
    C: Deref<Target = Query<'w, 's, Q, (F, With<InMap>, With<Chunk>)>>,
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    type Item = <<Q as QueryData>::ReadOnly as WorldQuery>::Item<'i>;

    #[allow(clippy::while_let_on_iterator)]
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(target) = self.coord_iter.next() {
            let tile = self.chunk_q.get_at(target);
            if tile.is_some() {
                return tile;
            }
        }

        None
    }
}

/// Iterates over a range of chunks using chunk coordinates.
pub struct ChunkQueryIterMut<'i, 'a, C, const N: usize> {
    coord_iter: CoordIterator<N>,
    chunk_q: &'i ChunkQuery<'a, C, N>,
}

impl<'i, 'a, C, const N: usize> ChunkQueryIterMut<'i, 'a, C, N> {
    fn new(chunk_q: &'i ChunkQuery<'a, C, N>, corner_1: [i32; N], corner_2: [i32; N]) -> Self {
        Self {
            chunk_q,
            coord_iter: CoordIterator::new(corner_1, corner_2),
        }
    }
}

impl<'i, 'a: 'i, 'w: 'a, 's: 'a, Q, F, C, const N: usize> Iterator
    for ChunkQueryIterMut<'i, 'a, C, N>
where
    C: DerefMut<Target = Query<'w, 's, Q, (F, With<InMap>, With<Chunk>)>>,
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    type Item = <Q as WorldQuery>::Item<'i>;

    #[allow(clippy::while_let_on_iterator)]
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(target) = self.coord_iter.next() {
            // SAFETY: Safe as long as the constructor requires a mutable reference
            let tile = unsafe { self.chunk_q.get_at_unchecked(target) };
            if tile.is_some() {
                return tile;
            }
        }

        None
    }
}
