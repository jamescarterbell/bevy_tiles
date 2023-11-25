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

#[derive(Component)]
pub(crate) struct MapLabel<L>(PhantomData<L>);

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
}

impl<const N: usize> Default for TileMap<N> {
    fn default() -> Self {
        Self {
            chunks: Default::default(),
        }
    }
}
