use bevy::{
    ecs::{component::Component, entity::Entity},
    prelude::Deref,
};

mod chunk_query;

pub use chunk_query::*;

/// An relation on chunks that point towards the map they are a part of.
#[derive(Component, Deref, Debug)]
pub struct InMap(pub(crate) Entity);

/// The coordinate of a given chunk.
/// # Note
#[derive(Component, Clone, Copy, Deref, Debug, PartialEq, Eq, Hash)]
pub struct ChunkCoord<const N: usize = 2>(pub(crate) [isize; N]);

/// Holds handles to all the tiles in a chunk.
/// # Note
/// Manually updating this value, adding it, or removing it from an entity may
/// cause issues, please only mutate chunk information via commands.
#[derive(Component, Deref, Debug)]
pub struct Chunk(pub(crate) Vec<Option<Entity>>);

impl Chunk {
    pub(crate) fn new(chunk_size: usize) -> Self {
        Self(vec![None; chunk_size])
    }

    pub(crate) fn get(&self, tile_i: usize) -> Option<Entity> {
        self.0.get(tile_i).cloned().flatten()
    }
}
