use bevy::prelude::*;

mod tile_query;

pub use tile_query::*;

/// The index of a tile in a given chunk.
#[derive(Component, Deref, Debug)]
pub struct TileIndex(pub(crate) usize);

/// The coordinate of a tile in a given map.
#[derive(Component, Deref, Debug)]
pub struct TileCoord<const N: usize = 2>(pub(crate) [isize; N]);

/// A relation on tiles that point towards the chunk they are a part of.
#[derive(Component, Deref, Debug)]
pub struct InChunk(pub(crate) Entity);
