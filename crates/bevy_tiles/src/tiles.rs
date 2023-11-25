use aery::prelude::*;
use bevy::prelude::*;
use std::ops::Deref;

mod tile_query;

pub use tile_query::*;

/// The index of a tile in a given chunk.
/// # Note
/// Manually updating this value, adding it, or removing it from an entity may
/// cause issues, please only mutate tile information via commands.
#[derive(Component, Debug)]
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
/// # Note
/// Manually updating this value, adding it, or removing it from an entity may
/// cause issues, please only mutate tile information via commands.
#[derive(Component, Debug)]
pub struct TileCoord<const N: usize = 2>([isize; N]);

impl<const N: usize> TileCoord<N> {
    pub(crate) fn new(value: [isize; N]) -> Self {
        Self(value)
    }
}

impl<const N: usize> Deref for TileCoord<N> {
    type Target = [isize; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// An aery relation on tiles that point towards the chunk they are a part of.
#[derive(Relation)]
#[aery(Recursive)]
pub struct InChunk<L, const N: usize>(std::marker::PhantomData<L>);
