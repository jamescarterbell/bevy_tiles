use bevy::{
    ecs::{
        entity::Entity,
        query::{QueryData, QueryFilter, With, WorldQuery},
        system::SystemParam,
    },
    prelude::Query,
};
use bevy_tiles::{
    chunks::{ChunkMapQuery, ChunkQuery, InMap},
    coords::{
        calculate_chunk_coordinate, calculate_tile_coordinate, calculate_tile_index,
        max_tile_index, CoordIterator,
    },
    queries::TileDataQuery,
};

use crate::{entity_tile::InChunk, EntityTile};

/// Used to query individual tiles from a tile map.
/// This query also implicitly queries chunks and maps
/// in order to properly resolve tiles.
#[derive(SystemParam)]
pub struct TileEntityMapQuery<'w, 's, Q, F, const N: usize = 2>
where
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    tile_q: Query<'w, 's, Q, (F, With<InChunk>)>,
    chunk_q: ChunkMapQuery<'w, 's, <EntityTile as TileDataQuery>::Source, With<InMap>, N>,
}

impl<'w, 's, Q, F, const N: usize> TileEntityMapQuery<'w, 's, Q, F, N>
where
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    /// Gets the query for a given map.
    pub fn get_map(
        &self,
        map_id: Entity,
    ) -> Option<TileEntityQuery<'_, '_, 's, Q::ReadOnly, F, N>> {
        let chunk_q = self.chunk_q.get_map(map_id)?;

        Some(TileEntityQuery {
            tile_q: self.tile_q.to_readonly(),
            chunk_q,
        })
    }

    /// Gets the query for a given map.
    pub fn get_map_mut(&mut self, map_id: Entity) -> Option<TileEntityQuery<'_, '_, 's, Q, F, N>> {
        let chunk_q = self.chunk_q.get_map_mut(map_id)?;

        Some(TileEntityQuery {
            tile_q: self.tile_q.reborrow(),
            chunk_q,
        })
    }
}

/// Queries a particular tilemap.
pub struct TileEntityQuery<'a, 'w, 's, Q, F, const N: usize = 2>
where
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    tile_q: Query<'w, 's, Q, (F, With<InChunk>)>,
    chunk_q: ChunkQuery<'a, 'w, 's, <EntityTile as TileDataQuery>::Source, With<InMap>, N>,
}

impl<'a, 'w, 's, Q, F, const N: usize> TileEntityQuery<'a, 'w, 's, Q, F, N>
where
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    /// Get the readonly variant of this query.
    pub fn to_readonly(&self) -> TileEntityQuery<'_, '_, 's, Q::ReadOnly, F, N> {
        TileEntityQuery {
            tile_q: self.tile_q.to_readonly(),
            chunk_q: self.chunk_q.to_readonly(),
        }
    }

    /// Get the readonly variant of this query.
    pub fn reborrow(&mut self) -> TileEntityQuery<'_, '_, 's, Q, F, N> {
        TileEntityQuery {
            tile_q: self.tile_q.reborrow(),
            chunk_q: self.chunk_q.reborrow(),
        }
    }

    /// Gets the readonly query item for the given tile.
    pub fn get_at(
        &self,
        tile_c: impl Into<[i32; N]>,
    ) -> Option<<Q::ReadOnly as WorldQuery>::Item<'_>> {
        let tile_c = tile_c.into();
        let tile_i = calculate_tile_index(tile_c, self.chunk_q.map.get_chunk_size());
        let chunk_c = calculate_chunk_coordinate(tile_c, self.chunk_q.map.get_chunk_size());
        let chunk_e = self.chunk_q.get_at(chunk_c)?;
        let tile_id = chunk_e.get(tile_i)?;
        self.tile_q.get(**tile_id).ok()
    }

    /// Gets the query item for the given tile.
    pub fn get_at_mut(
        &mut self,
        tile_c: impl Into<[i32; N]>,
    ) -> Option<<Q as WorldQuery>::Item<'_>> {
        let tile_c = tile_c.into();
        let tile_i = calculate_tile_index(tile_c, self.chunk_q.map.get_chunk_size());
        let chunk_c = calculate_chunk_coordinate(tile_c, self.chunk_q.map.get_chunk_size());
        let chunk_e = self.chunk_q.get_at(chunk_c)?;
        let tile_id = chunk_e.get(tile_i)?;
        self.tile_q.get_mut(**tile_id).ok()
    }

    /// Gets the query item for the given tile.
    /// # Safety
    /// This function makes it possible to violate Rust's aliasing guarantees: please use responsibly.
    pub unsafe fn get_at_unchecked(
        &self,
        tile_c: impl Into<[i32; N]>,
    ) -> Option<<Q as WorldQuery>::Item<'_>> {
        let tile_c = tile_c.into();
        let tile_i = calculate_tile_index(tile_c, self.chunk_q.map.get_chunk_size());
        let chunk_c = calculate_chunk_coordinate(tile_c, self.chunk_q.map.get_chunk_size());
        let chunk_e = self.chunk_q.get_at(chunk_c)?;
        let tile_id = chunk_e.get(tile_i)?;
        self.tile_q.get_unchecked(**tile_id).ok()
    }

    /// Iterate over all the tiles in a given space, starting at `corner_1`
    /// inclusive over `corner_2`
    pub fn iter_in(
        &self,
        corner_1: impl Into<[i32; N]>,
        corner_2: impl Into<[i32; N]>,
    ) -> TileEntityQueryIter<'_, 's, Q::ReadOnly, F, N> {
        let corner_1 = corner_1.into();
        let corner_2 = corner_2.into();
        // SAFETY: This thing is uses manual mem management
        unsafe { TileEntityQueryIter::from_owned(self.to_readonly(), corner_1, corner_2) }
    }

    /// Iterate over all the tiles in a given space, starting at `corner_1`
    /// inclusive over `corner_2`
    pub fn iter_in_mut(
        &mut self,
        corner_1: impl Into<[i32; N]>,
        corner_2: impl Into<[i32; N]>,
    ) -> TileEntityQueryIter<'_, 's, Q, F, N> {
        let corner_1 = corner_1.into();
        let corner_2 = corner_2.into();
        // SAFETY: This thing is uses manual mem management
        unsafe { TileEntityQueryIter::from_owned(self.reborrow(), corner_1, corner_2) }
    }

    /// Iter all tiles in a given chunk.
    /// # Note
    /// The coordinates for this function are givne in chunk coordinates.
    pub fn iter_in_chunk(
        &self,
        chunk_c: impl Into<[i32; N]>,
    ) -> TileEntityQueryIter<'_, 's, Q::ReadOnly, F, N> {
        let chunk_c = chunk_c.into();
        let chunk_size = self.chunk_q.map.get_chunk_size();
        // Get corners of chunk
        let corner_1 = calculate_tile_coordinate(chunk_c, 0, chunk_size);
        let corner_2 =
            calculate_tile_coordinate(chunk_c, max_tile_index::<N>(chunk_size), chunk_size);
        // Todo: just read the vector directly essentially
        self.iter_in(corner_1, corner_2)
    }

    /// Iter all tiles in the chunks in the given range.
    /// # Note
    /// The coordinates for this function are givne in chunk coordinates.
    pub fn iter_in_chunks(
        &mut self,
        chunk_c_1: impl Into<[i32; N]>,
        chunk_c_2: impl Into<[i32; N]>,
    ) -> TileEntityQueryIter<'_, 's, Q::ReadOnly, F, N> {
        let chunk_c_1 = chunk_c_1.into();
        let chunk_c_2 = chunk_c_2.into();
        let chunk_size = self.chunk_q.map.get_chunk_size();
        // Get corners of chunk
        let corner_1 = calculate_tile_coordinate(chunk_c_1, 0, chunk_size);
        let corner_2 =
            calculate_tile_coordinate(chunk_c_2, max_tile_index::<N>(chunk_size), chunk_size);
        // Todo: just read the vector directly essentially
        self.iter_in(corner_1, corner_2)
    }

    /// Iter all tiles in the chunks in the given range.
    /// # Note
    /// The coordinates for this function are givne in chunk coordinates.
    pub fn iter_in_chunks_mut(
        &mut self,
        chunk_c_1: impl Into<[i32; N]>,
        chunk_c_2: impl Into<[i32; N]>,
    ) -> TileEntityQueryIter<'_, 's, Q, F, N> {
        let chunk_c_1 = chunk_c_1.into();
        let chunk_c_2 = chunk_c_2.into();
        let chunk_size = self.chunk_q.map.get_chunk_size();
        // Get corners of chunk
        let corner_1 = calculate_tile_coordinate(chunk_c_1, 0, chunk_size);
        let corner_2 =
            calculate_tile_coordinate(chunk_c_2, max_tile_index::<N>(chunk_size), chunk_size);

        self.iter_in_mut(corner_1, corner_2)
    }

    /// Iter all tiles in a given chunk.
    /// # Note
    /// The coordinates for this function are givne in chunk coordinates.
    pub fn iter_in_chunk_mut(
        &mut self,
        chunk_c: impl Into<[i32; N]>,
    ) -> TileEntityQueryIter<'_, 's, Q, F, N> {
        let chunk_c = chunk_c.into();
        let chunk_size = self.chunk_q.map.get_chunk_size();
        // Get corners of chunk
        let corner_1 = calculate_tile_coordinate(chunk_c, 0, chunk_size);
        let corner_2 =
            calculate_tile_coordinate(chunk_c, max_tile_index::<N>(chunk_size), chunk_size);

        self.iter_in_mut(corner_1, corner_2)
    }
}

// Everything below here is astoundingly unsafe but I think it's sound
// If we're iterating over a readonly query, we're manually managing the lifetime of
// the readonly query by making the TileQueryIter own it as a reference.

/// Iterates over all the tiles in a region.
pub struct TileEntityQueryIter<'a, 's, Q, F, const N: usize>
where
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    coord_iter: CoordIterator<N>,
    tile_q: TileEntityQuery<'a, 'a, 's, Q, F, N>,
}
impl<'a, 's, Q, F, const N: usize> TileEntityQueryIter<'a, 's, Q, F, N>
where
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    unsafe fn from_owned(
        tile_q: TileEntityQuery<'a, 'a, 's, Q, F, N>,
        corner_1: [i32; N],
        corner_2: [i32; N],
    ) -> Self {
        Self {
            tile_q,
            coord_iter: CoordIterator::new(corner_1, corner_2),
        }
    }
}

impl<'a, 's, Q, F, const N: usize> Iterator for TileEntityQueryIter<'a, 's, Q, F, N>
where
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    type Item = <Q as WorldQuery>::Item<'a>;

    #[allow(clippy::while_let_on_iterator)]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(target) = self.coord_iter.next() {
            // SAFETY: Same as below.
            let tile = unsafe { self.tile_q.get_at_unchecked(target) };
            if tile.is_some() {
                // SAFETY: Since this is always tied to the lifetime of the reference we are reborrowing query from, we're just
                // telling the compiler here that we understand this particular item is pointing to something above this iterator.
                // Even if we drop the iterator, we can't create a new one or mutably borrow the underlying query again, since
                // this returned itemed will keep the original borrow used to make the iterator alive in the mind of the compiler.
                return unsafe {
                    std::mem::transmute::<
                        std::option::Option<<Q as WorldQuery>::Item<'_>>,
                        std::option::Option<<Q as WorldQuery>::Item<'_>>,
                    >(tile)
                };
            }
        }

        None
    }
}
