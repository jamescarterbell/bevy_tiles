use bevy::{
    ecs::{component::Component, entity::Entity, system::Resource},
    prelude::{Deref, DerefMut},
    transform::components::GlobalTransform,
};
use crossbeam::queue::SegQueue;
use dashmap::DashMap;

use super::{TileGridSize, TileMapRenderer, TileSize};

#[derive(Default, Resource, Deref, DerefMut)]
pub struct SavedMaps(DashMap<Entity, MapInfo>);

#[derive(Default, Component, Deref, DerefMut)]
pub struct MapChunks(SegQueue<Entity>);

#[derive(Clone, Component)]
pub struct MapInfo {
    pub chunk_size: u32,
    pub tile_map_renderer: TileMapRenderer,
    pub tile_size: TileSize,
    pub grid_size: TileGridSize,
    pub transform: GlobalTransform,
}
