use bevy::prelude::*;
use std::ops::Deref;

mod tile_query;

pub use tile_query::*;

/// The index of a tile in a given chunk.
#[derive(Debug)]
pub struct TileIndex(usize);

impl From<usize> for TileIndex {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl Deref for TileIndex {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// The coordinate of a tile in a given map.
#[derive(Debug)]
pub struct TileCoord<const N: usize = 2>([isize; N]);

impl<const N: usize> TileCoord<N> {
    pub(crate) fn new(value: [isize; N]) -> Self {
        Self(value)
    }
}

impl<const N: usize> From<[isize; N]> for TileCoord<N> {
    fn from(value: [isize; N]) -> Self {
        Self(value)
    }
}

impl<const N: usize> Deref for TileCoord<N> {
    type Target = [isize; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A relation on tiles that point towards the chunk they are a part of.
#[derive(Component)]
pub struct InChunk(pub(crate) Entity);

impl InChunk {
    /// Get the referenced chunk entity.
    pub fn get(&self) -> Entity {
        self.0
    }
}
