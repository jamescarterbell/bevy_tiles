use std::ops::Deref;

use aery::prelude::{CleanupPolicy, Relation};
use bevy::ecs::{component::Component, entity::Entity};

mod chunk_query;

pub use chunk_query::*;

/// An aery relation on chunks that point towards the map they are a part of.
#[derive(Relation)]
#[aery(Recursive)]
pub struct InMap<L, const N: usize>(std::marker::PhantomData<L>);

/// The coordinate of a given chunk.
/// # Note
/// Right now, changes to this coordinate don't automatically update any information.
/// If you wish to move a chunk, add, or remove a chunk, please do so via commands.
/// Use this if you wish to track changes or other information.
#[derive(Component, Debug, PartialEq, Eq, Hash)]
pub struct ChunkCoord<const N: usize = 2>([isize; N]);

impl<const N: usize> From<[isize; N]> for ChunkCoord<N> {
    fn from(value: [isize; N]) -> Self {
        Self(value)
    }
}

impl<const N: usize> Deref for ChunkCoord<N> {
    type Target = [isize; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Holds handles to all the tiles in a chunk.
/// # Note
/// Manually updating this value, adding it, or removing it from an entity may
/// cause issues, please only mutate chunk information via commands.
#[derive(Component)]
pub struct Chunk {
    pub(crate) tiles: Vec<Option<Entity>>,
}

impl Chunk {
    pub(crate) fn new(chunk_size: usize) -> Self {
        Self {
            tiles: vec![None; chunk_size],
        }
    }
}
