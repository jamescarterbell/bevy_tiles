use bevy::{
    ecs::{
        entity::Entity,
        query::{QueryData, With, WorldQuery},
        system::SystemParam,
    },
    prelude::Query,
};

use crate::{
    chunks::{Chunk, InMap},
    coords::{calculate_tile_index, CoordIterator},
    maps::TileMap,
    queries::{TileData, TileDataQuery},
    utils::{Owm, Rop},
};

use super::TileCoord;

/// Used to query individual tiles from a tile map.
/// This query also implicitly queries chunks and maps
/// in order to properly resolve tiles.
#[derive(SystemParam)]
pub struct TileMapQuery<'w, 's, Q, const N: usize = 2>
where
    Q: TileData + 'static,
{
    chunk_q: Query<'w, 's, (<Q as TileDataQuery>::Source, &'static Chunk), With<InMap>>,
    map_q: Query<'w, 's, &'static TileMap<N>>,
}

impl<'w, 's, Q, const N: usize> TileMapQuery<'w, 's, Q, N>
where
    Q: TileData + 'static,
{
    /// Gets the query for a given map.
    pub fn get_map(&self, map_id: Entity) -> Option<TileQuery<'_, '_, 's, Q::ReadOnly, N>> {
        let map = self.map_q.get(map_id).ok()?;

        Some(TileQuery {
            chunk_q: Owm::Owned(self.chunk_q.to_readonly()),
            map,
        })
    }

    /// Gets the query for a given map.
    pub fn get_map_mut(&mut self, map_id: Entity) -> Option<TileQuery<'_, 'w, 's, Q, N>> {
        let map = self.map_q.get(map_id).ok()?;

        Some(TileQuery {
            chunk_q: Owm::Borrowed(&mut self.chunk_q),
            map,
        })
    }
}

/// Queries a particular tilemap.
pub struct TileQuery<'a, 'w, 's, Q, const N: usize = 2>
where
    Q: TileData + 'static,
{
    chunk_q: Owm<'a, Query<'w, 's, (<Q as TileDataQuery>::Source, &'static Chunk), With<InMap>>>,
    map: &'a TileMap<N>,
}

impl<'a, 'w, 's, Q, const N: usize> TileQuery<'a, 'w, 's, Q, N>
where
    Q: TileData + 'static,
{
    /// Get the readonly variant of this query.
    pub fn to_readonly(&self) -> TileQuery<'_, '_, 's, Q::ReadOnly, N> {
        TileQuery {
            chunk_q: Owm::Owned(self.chunk_q.to_readonly()),
            map: self.map,
        }
    }

    fn get_chunk_data(
        &self,
        tile_c: [i32; N],
    ) -> Option<<<<Q as TileDataQuery>::Source as QueryData>::ReadOnly as WorldQuery>::Item<'_>>
    {
        let chunk_id = self.map.get_from_tile(TileCoord::<N>(tile_c))?;
        self.chunk_q.get(chunk_id).ok().map(|r| r.0)
    }

    fn get_chunk_data_mut(
        &mut self,
        tile_c: [i32; N],
    ) -> Option<<<Q as TileDataQuery>::Source as WorldQuery>::Item<'_>> {
        let chunk_id = self.map.get_from_tile(TileCoord::<N>(tile_c))?;
        self.chunk_q.get_mut(chunk_id).ok().map(|r| r.0)
    }

    unsafe fn get_chunk_data_unchecked(
        &self,
        tile_c: [i32; N],
    ) -> Option<<<Q as TileDataQuery>::Source as WorldQuery>::Item<'_>> {
        let chunk_id = self.map.get_from_tile(TileCoord::<N>(tile_c))?;
        self.chunk_q.get_unchecked(chunk_id).ok().map(|r| r.0)
    }

    /// Gets the readonly query item for the given tile.
    pub fn get_at(
        &self,
        tile_c: impl Into<[i32; N]>,
    ) -> Option<<<Q as TileData>::ReadOnly as TileDataQuery>::Item<'_>> {
        let tile_c = tile_c.into();
        let tile_i = calculate_tile_index(tile_c, self.map.get_chunk_size());
        let tile_e = self.get_chunk_data(tile_c)?;

        <<Q as TileData>::ReadOnly as TileDataQuery>::get(tile_e, tile_i)
    }

    /// Gets the query item for the given tile.
    /// # Safety
    /// This function makes it possible to violate Rust's aliasing guarantees: please use responsibly.
    pub unsafe fn get_at_unchecked(
        &self,
        tile_c: impl Into<[i32; N]>,
    ) -> Option<<Q as TileDataQuery>::Item<'_>> {
        let tile_c = tile_c.into();
        let tile_i = calculate_tile_index(tile_c, self.map.get_chunk_size());
        let tile_e = self.get_chunk_data_unchecked(tile_c)?;

        Q::get(tile_e, tile_i)
    }

    /// Iterate over all the tiles in a given space, starting at `corner_1`
    /// inclusive over `corner_2`
    pub fn iter_in(
        &self,
        corner_1: impl Into<[i32; N]>,
        corner_2: impl Into<[i32; N]>,
    ) -> TileQueryIter<'_, '_, '_, 's, Q::ReadOnly, N> {
        let corner_1 = corner_1.into();
        let corner_2 = corner_2.into();
        // SAFETY: This thing is uses manual mem management
        unsafe { TileQueryIter::from_owned(self.to_readonly(), corner_1, corner_2) }
    }

    // /// Iter all tiles in a given chunk.
    // /// # Note
    // /// The coordinates for this function are givne in chunk coordinates.
    // pub fn iter_in_chunk(&self, chunk_c: impl Into<[i32; N]>) -> TileQueryIter<'_, 'a, T, C, N> {
    //     let chunk_c = chunk_c.into();
    //     let chunk_size = self.map.get_chunk_size();
    //     // Get corners of chunk
    //     let corner_1 = calculate_tile_coordinate(chunk_c, 0, chunk_size);
    //     let corner_2 =
    //         calculate_tile_coordinate(chunk_c, max_tile_index::<N>(chunk_size), chunk_size);
    //     // Create tile iter
    //     TileQueryIter::new(self, corner_1, corner_2)
    // }

    // /// Iter all tiles in the chunks in the given range.
    // /// # Note
    // /// The coordinates for this function are givne in chunk coordinates.
    // pub fn iter_in_chunks(
    //     &mut self,
    //     chunk_c_1: impl Into<[i32; N]>,
    //     chunk_c_2: impl Into<[i32; N]>,
    // ) -> TileQueryIter<'_, 'a, 'w, 's, T, C, N> {
    //     let chunk_c_1 = chunk_c_1.into();
    //     let chunk_c_2 = chunk_c_2.into();
    //     let chunk_size = self.map.get_chunk_size();
    //     // Get corners of chunk
    //     let corner_1 = calculate_tile_coordinate(chunk_c_1, 0, chunk_size);
    //     let corner_2 =
    //         calculate_tile_coordinate(chunk_c_2, max_tile_index::<N>(chunk_size), chunk_size);
    //     // Create tile iter
    //     TileQueryIter::new(self, corner_1, corner_2)
    // }

    /// Gets the query item for the given tile.
    pub fn get_at_mut(
        &mut self,
        tile_c: impl Into<[i32; N]>,
    ) -> Option<<Q as TileDataQuery>::Item<'_>> {
        let tile_c = tile_c.into();
        let tile_i = calculate_tile_index(tile_c, self.map.get_chunk_size());
        let tile_e = self.get_chunk_data_mut(tile_c)?;

        Q::get(tile_e, tile_i)
    }

    // /// Iter all tiles in the chunks in the given range.
    // /// # Note
    // /// The coordinates for this function are givne in chunk coordinates.
    // pub fn iter_in_chunks_mut(
    //     &mut self,
    //     chunk_c_1: impl Into<[i32; N]>,
    //     chunk_c_2: impl Into<[i32; N]>,
    // ) -> TileQueryIterMut<'_, 'a, T, C, N> {
    //     let chunk_c_1 = chunk_c_1.into();
    //     let chunk_c_2 = chunk_c_2.into();
    //     let chunk_size = self.map.get_chunk_size();
    //     // Get corners of chunk
    //     let corner_1 = calculate_tile_coordinate(chunk_c_1, 0, chunk_size);
    //     let corner_2 =
    //         calculate_tile_coordinate(chunk_c_2, max_tile_index::<N>(chunk_size), chunk_size);

    //     TileQueryIterMut::new(self, corner_1, corner_2)
    // }

    // /// Iter all tiles in a given chunk.
    // /// # Note
    // /// The coordinates for this function are givne in chunk coordinates.
    // pub fn iter_in_chunk_mut(
    //     &mut self,
    //     chunk_c: impl Into<[i32; N]>,
    // ) -> TileQueryIterMut<'_, 'a, T, C, N> {
    //     let chunk_c = chunk_c.into();
    //     let chunk_size = self.map.get_chunk_size();
    //     // Get corners of chunk
    //     let corner_1 = calculate_tile_coordinate(chunk_c, 0, chunk_size);
    //     let corner_2 =
    //         calculate_tile_coordinate(chunk_c, max_tile_index::<N>(chunk_size), chunk_size);

    //     TileQueryIterMut::new(self, corner_1, corner_2)
    // }

    /// Iterate over all the tiles in a given space, starting at `corner_1`
    /// inclusive over `corner_2`
    pub fn iter_in_mut(
        &mut self,
        corner_1: impl Into<[i32; N]>,
        corner_2: impl Into<[i32; N]>,
    ) -> TileQueryIter<'_, 'a, 'w, 's, Q, N> {
        let corner_1 = corner_1.into();
        let corner_2 = corner_2.into();
        // SAFETY: This thing is uses manual mem management
        unsafe { TileQueryIter::from_ref(self, corner_1, corner_2) }
    }
}

// Everything below here is astoundingly unsafe but I think it's sound
// If we're iterating over a readonly query, we're manually managing the lifetime of
// the readonly query by making the TileQueryIter own it as a reference.

/// Iterates over all the tiles in a region.
pub struct TileQueryIter<'i, 'a, 'w, 's, Q, const N: usize>
where
    Q: TileData + 'static,
{
    coord_iter: CoordIterator<N>,
    tile_q: Rop<'i, TileQuery<'a, 'w, 's, Q, N>>,
}
impl<'i, 'a, 'w, 's, Q, const N: usize> TileQueryIter<'i, 'a, 'w, 's, Q, N>
where
    Q: TileData + 'static,
{
    unsafe fn from_ref(
        tile_q: &'i TileQuery<'a, 'w, 's, Q, N>,
        corner_1: [i32; N],
        corner_2: [i32; N],
    ) -> Self {
        Self {
            tile_q: Rop::from_ref(tile_q),
            coord_iter: CoordIterator::new(corner_1, corner_2),
        }
    }

    unsafe fn from_owned(
        tile_q: TileQuery<'a, 'w, 's, Q, N>,
        corner_1: [i32; N],
        corner_2: [i32; N],
    ) -> Self {
        Self {
            tile_q: Rop::from_owned(tile_q),
            coord_iter: CoordIterator::new(corner_1, corner_2),
        }
    }
}

impl<'i, 'a, 'w, 's, Q, const N: usize> Iterator for TileQueryIter<'i, 'a, 'w, 's, Q, N>
where
    Q: TileData + 'static,
{
    type Item = <Q as TileDataQuery>::Item<'i>;

    #[allow(clippy::while_let_on_iterator)]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(target) = self.coord_iter.next() {
            // SAFETY: It might not be
            let tile = unsafe { self.tile_q.get().get_at_unchecked(target) };
            if tile.is_some() {
                return tile;
            }
        }

        None
    }
}
