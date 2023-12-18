use bevy::{
    ecs::{component::Component, entity::Entity, system::Resource},
    prelude::{Deref, DerefMut},
};
use bevy_tiles::chunks::ChunkCoord;
use dashmap::DashMap;

use crate::bindings::ChunkBuffer;

#[derive(Default, Resource, Deref, DerefMut)]
pub struct SavedChunks(DashMap<Entity, ChunkBuffer>);

#[derive(Component, Deref)]
pub struct BatchSize(pub u32);

/// Holds a reference to the batch this chunk is in
#[derive(Component, Deref, Clone)]
pub struct ChunkBatch(pub Entity);

/// Data needed to render a chunk in the batched chunk rendering pipeline.
/// This needs to be able to be instantiated in the extract stage and should
/// not have knowledge of the batch it's in.
#[derive(Debug, Component)]
pub struct ChunkUniforms {
    pub chunk_coord: ChunkCoord,
    pub tile_instances: Option<Vec<u32>>,
}
