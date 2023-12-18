use bevy::ecs::{entity::Entity, query::With, system::ResMut, world::World};

use crate::{
    bindings::ChunkBuffer,
    chunk::internal::SavedChunks,
    maps::internal::{MapInfo, SavedMaps},
};

pub fn save_chunks(mut world: &mut World) {
    // Get the map id's we need
    let map_ids: Vec<Entity> = world
        .query_filtered::<Entity, With<MapInfo>>()
        .iter(world)
        .collect();

    // Get the chunk id's we need
    let chunk_ids: Vec<Entity> = world
        .query_filtered::<Entity, With<ChunkBuffer>>()
        .iter(world)
        .collect();

    // Remove the map entities and put them into our hashmap
    let mut saved_maps = SavedMaps::default();
    for map_id in map_ids.into_iter() {
        let mut map = world.entity_mut(map_id);
        saved_maps.insert(map_id, map.take::<MapInfo>().unwrap());
    }

    // Remove the chunk entities and put them into our hashmap
    let mut saved_chunks = SavedChunks::default();
    for chunk_id in chunk_ids.into_iter() {
        let mut chunk = world.entity_mut(chunk_id);
        saved_chunks.insert(chunk_id, chunk.take::<ChunkBuffer>().unwrap());
    }

    world.insert_resource(saved_maps);
    world.insert_resource(saved_chunks);
}
