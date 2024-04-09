use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

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
    chunks::ChunkCoord,
    maps::TileMap,
    prelude::{Chunk, InMap},
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
    ) -> Option<ChunkQuery<BorrowedChunkQueries<'_, 'w, 's, Q, F, N>>> {
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
    ) -> Option<ChunkQuery<MutableBorrowedChunkQueries<'_, 'w, 's, Q, F, N>>> {
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
pub struct ChunkQuery<'a, W>
where
    W: BorrowedChunkQueryTypes<'a>,
{
    chunk_q: W::ChunkQuery,
    map: W::Map,
}

impl<'a, 'w: 'a, 's: 'a, Q, F, W, const N: usize> ChunkQuery<'a, W>
where
    W: BorrowedChunkQueryTypes<'a>,
    <W as BorrowedChunkQueryTypes<'a>>::ChunkQuery:
        Deref<Target = Query<'w, 's, Q, (F, With<InMap>, With<Chunk>)>>,
    <W as BorrowedChunkQueryTypes<'a>>::Map: Deref<Target = TileMap<N>>,
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    /// Get's the readonly query item for the given tile.
    /// # Note
    /// Coordinates are for these calls are in chunk coordinates.
    #[inline]
    pub fn get_at(
        &self,
        chunk_c: [isize; N],
    ) -> Option<<<Q as QueryData>::ReadOnly as WorldQuery>::Item<'_>> {
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
        chunk_c: [isize; N],
    ) -> Option<<Q as WorldQuery>::Item<'_>> {
        let chunk_id = self.map.get_from_chunk(ChunkCoord(chunk_c))?;

        self.chunk_q.get_unchecked(chunk_id).ok()
    }

    // /// Iterate over all the chunks in a given space, starting at `corner_1`
    // /// inclusive over `corner_2`
    // /// # Note
    // /// Coordinates are for these calls are in chunk coordinates.
    // #[inline]
    // pub fn iter_in(
    //     &self,
    //     corner_1: [isize; N],
    //     corner_2: [isize; N],
    // ) -> ChunkQueryIter<'_, 's, Q, F, N> {
    //     ChunkQueryIter::new(self, corner_1, corner_2)
    // }

    // /// Iterate over all the chunks in a given space, starting at `corner_1`
    // /// inclusive over `corner_2`.
    // /// # Note
    // /// Coordinates are for these calls are in chunk coordinates.
    // #[inline]
    // pub fn iter_in_mut(
    //     &mut self,
    //     corner_1: [isize; N],
    //     corner_2: [isize; N],
    // ) -> ChunkQueryIterMut<'_, 's, Q, F, N> {
    //     // SAFETY: Use case is safe since this is the mut version and the function signature
    //     // stops us from borrowing this mutably twice
    //     unsafe { ChunkQueryIterMut::new(self, corner_1, corner_2) }
    // }

    // /// Get the readonly version of this query.
    // #[inline]
    // pub fn to_readonly(&self) -> ChunkMapQuery<'_, 's, <Q as QueryData>::ReadOnly, F, N> {
    //     ChunkMapQuery::<<Q as QueryData>::ReadOnly, F, N> {
    //         chunk_q: self.chunk_q.to_readonly(),
    //         map: self.map,
    //     }
    // }
}

impl<'a, 'w: 'a, 's: 'a, Q, F, W, const N: usize> ChunkQuery<'a, W>
where
    W: BorrowedChunkQueryTypes<'a>,
    <W as BorrowedChunkQueryTypes<'a>>::ChunkQuery:
        DerefMut<Target = Query<'w, 's, Q, (F, With<InMap>, With<Chunk>)>>,
    <W as BorrowedChunkQueryTypes<'a>>::Map: Deref<Target = TileMap<N>>,
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    /// Get's the query item for the given tile.
    /// # Note
    /// Coordinates are for these calls are in chunk coordinates.
    #[inline]
    pub fn get_at_mut(&mut self, chunk_c: [isize; N]) -> Option<<Q as WorldQuery>::Item<'_>> {
        let chunk_id = self.map.get_from_chunk(ChunkCoord(chunk_c))?;

        self.chunk_q.get_mut(chunk_id).ok()
    }
}

/// Borrowed types from a [ChunkQuery] needed to construct a [ChunkMapQuery]
pub struct BorrowedChunkQueries<'a, 'w: 'a, 's: 'a, Q, F, const N: usize>
where
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    phantom: PhantomData<(&'a Q, &'w F, &'s Q)>,
}

impl<'a, 'w: 'a, 's: 'a, Q, F, const N: usize> BorrowedChunkQueryTypes<'a>
    for BorrowedChunkQueries<'a, 'w, 's, Q, F, N>
where
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    type ChunkQuery = &'a Query<'w, 's, Q, (F, With<InMap>, With<Chunk>)>;

    type Map = &'a TileMap<N>;
}

/// Mutable borrowed types from a [ChunkQuery] needed to construct a mutable [ChunkMapQuery]
pub struct MutableBorrowedChunkQueries<'a, 'w: 'a, 's: 'a, Q, F, const N: usize>
where
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    phantom: PhantomData<(&'a Q, &'w F, &'s Q)>,
}

impl<'a, 'w: 'a, 's: 'a, Q, F, const N: usize> BorrowedChunkQueryTypes<'a>
    for MutableBorrowedChunkQueries<'a, 'w, 's, Q, F, N>
where
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    type ChunkQuery = &'a mut Query<'w, 's, Q, (F, With<InMap>, With<Chunk>)>;

    type Map = &'a TileMap<N>;
}

/// Describes the types used to construct a query, mainly needed to reduce code duplication.
pub trait BorrowedChunkQueryTypes<'a> {
    /// Query for chunks.
    type ChunkQuery;
    /// The map used.
    type Map;
}

// /// Iterates over a range of chunks using chunk coordinates.
// pub struct ChunkQueryIter<'w, 's, Q, F, const N: usize>
// where
//     Q: QueryData + 'static,
//     F: QueryFilter + 'static,
// {
//     coord_iter: CoordIterator<N>,
//     chunk_q: &'w ChunkMapQuery<'w, 's, Q, F, N>,
// }

// impl<'w, 's, Q, F, const N: usize> ChunkQueryIter<'w, 's, Q, F, N>
// where
//     Q: QueryData + 'static,
//     F: QueryFilter + 'static,
// {
//     /// # Safety
//     /// This iterator uses unchecked get's to get around some lifetime issue I don't understand yet.
//     /// Due to this, you should only call this constructor from a context where the query is actually
//     /// borrowed mutabley.
//     fn new(
//         chunk_q: &'w ChunkMapQuery<'w, 's, Q, F, N>,
//         corner_1: [isize; N],
//         corner_2: [isize; N],
//     ) -> Self {
//         Self {
//             chunk_q,
//             coord_iter: CoordIterator::new(corner_1, corner_2),
//         }
//     }
// }

// impl<'w, 's, Q, F, const N: usize> Iterator for ChunkQueryIter<'w, 's, Q, F, N>
// where
//     Q: QueryData + 'static,
//     F: QueryFilter + 'static,
// {
//     type Item = <<Q as QueryData>::ReadOnly as WorldQuery>::Item<'w>;

//     #[allow(clippy::while_let_on_iterator)]
//     #[inline]
//     fn next(&mut self) -> Option<Self::Item> {
//         while let Some(target) = self.coord_iter.next() {
//             let tile = self.chunk_q.get_at(target);
//             if tile.is_some() {
//                 return tile;
//             }
//         }

//         None
//     }
// }

// /// Iterates over a range of chunks mutably using chunk coordinates.
// /// # Note
// /// Due to weird borrow checker stuff, this is a seperate struct.
// /// In the future, we may find a way to combine the iterators.
// /// ```compile_fail
// ///# // Because we're using unsafe, we need to make sure we don't mutabley alias.
// ///# fn multiple_iter_mut(mut tile_query: ChunkQuery<TestLayer, ()>) {
// ///#     let mut iter_1 = tile_query.iter_in([0, 0], [3, 3]);
// ///#     let mut iter_2 = tile_query.iter_in_mut([0, 0], [3, 3]);
// ///#     let _ = iter_1.next();
// ///#     let _ = iter_2.next();
// ///# }
// /// ```
// pub struct ChunkQueryIterMut<'w, 's, Q, F, const N: usize>
// where
//     Q: QueryData + 'static,
//     F: QueryFilter + 'static,
// {
//     coord_iter: CoordIterator<N>,
//     chunk_q: &'w ChunkMapQuery<'w, 's, Q, F, N>,
// }

// impl<'w, 's, Q, F, const N: usize> ChunkQueryIterMut<'w, 's, Q, F, N>
// where
//     Q: QueryData + 'static,
//     F: QueryFilter + 'static,
// {
//     /// # Safety
//     /// This iterator uses unchecked get's to get around some lifetime issue I don't understand yet.
//     /// Due to this, you should only call this constructor from a context where the query is actually
//     /// borrowed mutabley.
//     unsafe fn new(
//         chunk_q: &'w ChunkMapQuery<'w, 's, Q, F, N>,
//         corner_1: [isize; N],
//         corner_2: [isize; N],
//     ) -> Self {
//         Self {
//             chunk_q,
//             coord_iter: CoordIterator::new(corner_1, corner_2),
//         }
//     }
// }

// impl<'w, 's, Q, F, const N: usize> Iterator for ChunkQueryIterMut<'w, 's, Q, F, N>
// where
//     Q: QueryData + 'static,
//     F: QueryFilter + 'static,
// {
//     type Item = <Q as WorldQuery>::Item<'w>;

//     #[allow(clippy::while_let_on_iterator)]
//     #[inline]
//     fn next(&mut self) -> Option<Self::Item> {
//         while let Some(target) = self.coord_iter.next() {
//             // SAFETY: This fixes some lifetime issue that I'm not sure I understand quite yet, will do testing
//             let tile = unsafe { self.chunk_q.get_at_unchecked(target) };
//             if tile.is_some() {
//                 return tile;
//             }
//         }

//         None
//     }
// }
