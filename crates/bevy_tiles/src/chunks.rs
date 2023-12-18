use std::ops::Deref;

use bevy::{
    ecs::{component::Component, entity::Entity},
    math::Vec2,
};

mod chunk_query;

pub use chunk_query::*;

/// An relation on chunks that point towards the map they are a part of.
#[derive(Component, Clone)]
pub struct InMap(pub(crate) Entity);

impl InMap {
    /// Get the entity this chunk is in
    pub fn get(&self) -> Entity {
        self.0
    }
}

/// The coordinate of a given chunk.
/// # Note
/// Right now, changes to this coordinate don't automatically update any information.
/// If you wish to move a chunk, add, or remove a chunk, please do so via commands.
/// Use this if you wish to track changes or other information.
#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy)]
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

impl From<&ChunkCoord<2>> for Vec2 {
    fn from(value: &ChunkCoord<2>) -> Self {
        Vec2::new(value.0[0] as f32, value.0[1] as f32)
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

    /// Gets the total number of tiles this chunk can hold.
    pub fn total_size(&self) -> usize {
        self.tiles.len()
    }

    /// Gets a readonly reference to the underlying chunk data.
    pub fn get_tiles(&self) -> &Vec<Option<Entity>> {
        &self.tiles
    }
}
