use bevy::{
    ecs::{component::Component, entity::Entity},
    math::{IVec2, IVec3},
    prelude::Deref,
};

mod chunk_query;

pub use chunk_query::*;

/// An relation on chunks that point towards the map they are a part of.
/// # Note:
/// It probably won't break anything to manually copy this
/// to put it on your own entities, but this is only accurate
/// when mutated by the plugin.
#[derive(Component, Clone, Copy, Deref, Debug)]
pub struct InMap(pub(crate) Entity);

/// The coordinate of a given chunk.
/// # Note:
/// It probably won't break anything to manually copy this
/// to put it on your own entities, but this is only accurate
/// when mutated by the plugin.
#[derive(Component, Deref, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChunkCoord<const N: usize>(pub(crate) [i32; N]);

impl From<IVec2> for ChunkCoord<2> {
    fn from(value: IVec2) -> Self {
        Self(value.into())
    }
}

impl From<IVec3> for ChunkCoord<3> {
    fn from(value: IVec3) -> Self {
        Self(value.into())
    }
}

/// Holds handles to all the tiles in a chunk.
#[derive(Component, Debug)]
pub struct Chunk(pub(crate) Vec<Option<Entity>>);

impl Chunk {
    pub(crate) fn new(chunk_size: usize) -> Self {
        Self(vec![None; chunk_size])
    }

    pub(crate) fn get(&self, tile_i: usize) -> Option<Entity> {
        self.0.get(tile_i).cloned().flatten()
    }
}
