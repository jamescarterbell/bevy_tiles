use std::any::TypeId;

use bevy::{
    ecs::{component::Component, entity::Entity},
    math::{IVec2, IVec3},
    prelude::Deref,
    utils::HashSet,
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
#[derive(Default, Component, Debug)]
pub struct Chunk;

/// Holds data for tiles in chunk.
#[derive(Component, Debug)]
pub struct ChunkData<T> {
    pub(crate) tiles: Vec<Option<T>>,
    pub(crate) count: usize,
}

impl<T> ChunkData<T> {
    /// Create a new ChunkData with a given size.
    pub fn new(chunk_size: usize) -> Self {
        let mut tiles = Vec::new();
        tiles.resize_with(chunk_size, || None);
        Self { tiles, count: 0 }
    }

    /// Get tile data at a given index.
    pub fn get(&self, tile_i: usize) -> Option<&T> {
        self.tiles.get(tile_i).and_then(|f| f.as_ref())
    }

    /// Get tile data at a given index.
    pub fn get_mut(&mut self, tile_i: usize) -> Option<&mut T> {
        self.tiles.get_mut(tile_i).and_then(|f| f.as_mut())
    }

    pub(crate) fn get_mut_raw(&mut self, tile_i: usize) -> &mut Option<T> {
        self.tiles.get_mut(tile_i).expect("Out of index {}")
    }

    /// Take the value from this index.
    pub fn take(&mut self, tile_i: usize) -> Option<T> {
        let removed = self.tiles.get_mut(tile_i)?.take();
        removed.is_some().then(|| self.count -= 1);
        removed
    }

    /// Insert the value at this index.
    pub fn insert(&mut self, tile_i: usize, value: T) -> Option<T> {
        let target = self.get_mut_raw(tile_i);
        let replaced = std::mem::replace(target, Some(value));
        replaced.is_none().then(|| self.count += 1);
        replaced
    }

    /// The current number of items.
    pub fn get_count(&self) -> usize {
        self.count
    }
}

/// Holds a registry of all data types on a chunk, used to decide
/// if a chunk deserves to live :).
#[derive(Component, Default, Debug)]
pub struct ChunkTypes(pub HashSet<TypeId>);
