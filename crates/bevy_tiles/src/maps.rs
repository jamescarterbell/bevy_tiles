use std::marker::PhantomData;

use bevy::{
    ecs::{component::Component, entity::Entity},
    utils::HashMap,
};

use crate::chunks::ChunkCoord;

/// Adds type level info on how a Tile Map should be treated.
pub trait TileMapLabel: Send + Sync + 'static {
    /// How many tiles per dimension a chunk in this map extends.
    const CHUNK_SIZE: usize;
}

/// Marks an entity as being part of the tilemap.
/// # Note
/// Manually updating this value, adding it, or removing it from an entity may
/// cause issues, please only mutate map information via commands.
#[derive(Component)]
pub struct MapLabel<L>(PhantomData<L>);

impl<L> Default for MapLabel<L> {
    fn default() -> Self {
        Self(Default::default())
    }
}

/// Holds handles to all the chunks in a map.
/// # Note
/// Manually updating this value, adding it, or removing it from an entity may
/// cause issues, please only mutate map information via commands.
#[derive(Component)]
pub struct TileMap<const N: usize = 2> {
    pub(crate) chunks: HashMap<ChunkCoord<N>, Entity>,
    /// The size of a chunk in one direction.
    pub chunk_size: usize,
}

impl<const N: usize> TileMap<N> {
    pub(crate) fn with_chunk_size(chunk_size: usize) -> Self {
        Self {
            chunks: Default::default(),
            chunk_size,
        }
    }

    /// Get readonly access to the chunk table
    pub fn get_chunks(&self) -> &HashMap<ChunkCoord<N>, Entity> {
        &self.chunks
    }
}
