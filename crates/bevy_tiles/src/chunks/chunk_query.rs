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
    utils::{Owm, Rop},
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
    pub fn get_map(&self, map_id: Entity) -> Option<ChunkQuery<'_, '_, 's, Q::ReadOnly, F, N>> {
        let map = self.map_q.get(map_id).ok()?;

        Some(ChunkQuery {
            chunk_q: Owm::Owned(self.chunk_q.to_readonly()),
            map,
        })
    }

    /// Gets the query for a given map.
    pub fn get_map_mut(&mut self, map_id: Entity) -> Option<ChunkQuery<'_, 'w, 's, Q, F, N>> {
        let map = self.map_q.get(map_id).ok()?;

        Some(ChunkQuery {
            chunk_q: Owm::Borrowed(&mut self.chunk_q),
            map,
        })
    }
}

/// Used to query chunks from a tile map.
/// This query also implicitly queries maps
/// in order to properly resolve chunks.
pub struct ChunkQuery<'a, 'w, 's, Q, F = (), const N: usize = 2>
where
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    chunk_q: Owm<'a, Query<'w, 's, Q, (F, With<InMap>, With<Chunk>)>>,
    map: &'a TileMap<N>,
}

impl<'a, 'w, 's, Q, F, const N: usize> ChunkQuery<'a, 'w, 's, Q, F, N>
where
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    /// Get the readonly variant of this query.
    pub fn to_readonly(&self) -> ChunkQuery<'_, '_, 's, Q::ReadOnly, F, N> {
        ChunkQuery {
            chunk_q: Owm::Owned(self.chunk_q.to_readonly()),
            map: self.map,
        }
    }

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
    ) -> ChunkQueryIter<'_, '_, '_, 's, Q::ReadOnly, F, N> {
        let corner_1 = corner_1.into();
        let corner_2 = corner_2.into();
        // SAFETY: This thing is uses manual mem management
        unsafe { ChunkQueryIter::from_owned(self.to_readonly(), corner_1, corner_2) }
    }

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
    ) -> ChunkQueryIter<'_, 'a, 'w, 's, Q, F, N> {
        let corner_1 = corner_1.into();
        let corner_2 = corner_2.into();
        // SAFETY: This thing is uses manual mem management
        unsafe { ChunkQueryIter::from_ref(self, corner_1, corner_2) }
    }
}
// Everything below here is astoundingly unsafe but I think it's sound
// If we're iterating over a readonly query, we're manually managing the lifetime of
// the readonly query by making the TileQueryIter own it as a reference.

/// Iterates over all the tiles in a region.
pub struct ChunkQueryIter<'i, 'a, 'w, 's, Q, F, const N: usize>
where
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    coord_iter: CoordIterator<N>,
    chunk_q: Rop<'i, ChunkQuery<'a, 'w, 's, Q, F, N>>,
}
impl<'i, 'a, 'w, 's, Q, F, const N: usize> ChunkQueryIter<'i, 'a, 'w, 's, Q, F, N>
where
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    unsafe fn from_ref(
        chunk_q: &'i ChunkQuery<'a, 'w, 's, Q, F, N>,
        corner_1: [i32; N],
        corner_2: [i32; N],
    ) -> Self {
        Self {
            chunk_q: Rop::from_ref(chunk_q),
            coord_iter: CoordIterator::new(corner_1, corner_2),
        }
    }

    unsafe fn from_owned(
        chunk_q: ChunkQuery<'a, 'w, 's, Q, F, N>,
        corner_1: [i32; N],
        corner_2: [i32; N],
    ) -> Self {
        Self {
            chunk_q: Rop::from_owned(chunk_q),
            coord_iter: CoordIterator::new(corner_1, corner_2),
        }
    }
}

impl<'i, 'a, 'w, 's, Q, F, const N: usize> Iterator for ChunkQueryIter<'i, 'a, 'w, 's, Q, F, N>
where
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    type Item = Q::Item<'i>;

    #[allow(clippy::while_let_on_iterator)]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(target) = self.coord_iter.next() {
            // SAFETY: It might not be
            let tile = unsafe { self.chunk_q.get().get_at_unchecked(target) };
            if tile.is_some() {
                return tile;
            }
        }

        None
    }
}

impl<'a, 'w, 's, Q, F, const N: usize> Deref for ChunkQuery<'a, 'w, 's, Q, F, N>
where
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    type Target = Query<'w, 's, Q, (F, With<InMap>, With<Chunk>)>;

    fn deref(&self) -> &Self::Target {
        &self.chunk_q
    }
}

impl<'a, 'w, 's, Q, F, const N: usize> DerefMut for ChunkQuery<'a, 'w, 's, Q, F, N>
where
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.chunk_q
    }
}
