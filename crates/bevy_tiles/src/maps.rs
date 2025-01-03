use bevy::{
    ecs::{component::Component, entity::Entity},
    prelude::{Deref, DerefMut},
    utils::HashMap,
};

use crate::{chunks::ChunkCoord, coords::calculate_chunk_coordinate};

/// Holds handles to all the chunks in a map.
/// # Note
/// Manually updating this value, adding it, or removing it from an entity may
/// cause issues, please only mutate map information via commands.
#[derive(Component)]
pub struct TileMap<const N: usize = 2> {
    chunks: HashMap<ChunkCoord<N>, Entity>,
    /// The size of a chunk in one direction.
    chunk_size: usize,
}

impl<const N: usize> TileMap<N> {
    pub(crate) fn with_chunk_size(chunk_size: usize) -> Self {
        Self {
            chunks: Default::default(),
            chunk_size,
        }
    }

    /// Gets the chunk entity from a tile coordinate.
    pub fn get_from_tile(&self, tile_c: impl Into<[i32; N]>) -> Option<Entity> {
        let chunk_c = calculate_chunk_coordinate(tile_c, self.chunk_size);
        self.chunks
            .get::<ChunkCoord<N>>(&ChunkCoord::<N>(chunk_c))
            .cloned()
    }

    /// Gets the chunk entity from a chunk coordinate.
    pub fn get_from_chunk(&self, chunk_c: ChunkCoord<N>) -> Option<Entity> {
        self.chunks.get::<ChunkCoord<N>>(&chunk_c).cloned()
    }

    /// Get readonly access to the chunk table.
    pub fn get_chunks(&self) -> &HashMap<ChunkCoord<N>, Entity> {
        &self.chunks
    }

    pub(crate) fn get_chunks_mut(&mut self) -> &mut HashMap<ChunkCoord<N>, Entity> {
        &mut self.chunks
    }

    /// Get the size of chunks in this tilemap.
    #[inline]
    pub fn get_chunk_size(&self) -> usize {
        self.chunk_size
    }
}

/// Marker component for whether or not this map should use transforms.
/// # Note:
/// Removing this does not remove the transforms from all the children of this map.
#[derive(Component, Copy, Clone, Debug)]
pub struct UseTransforms;

/// The size of a tile along each axis.  Add this to a [`TileMap`] for child chunks
/// and tiles to have proper spacing based on tile size.
#[derive(Component, Copy, Clone, Debug, Deref, DerefMut)]
pub struct TileDims<const N: usize>(pub [f32; N]);

/// The space between tiles along each axis.Add this to a [`TileMap`] for child chunks
/// and tiles to have proper spacing based on tile spacing.
#[derive(Component, Copy, Clone, Debug, Deref, DerefMut)]
pub struct TileSpacing<const N: usize>(pub [f32; N]);
