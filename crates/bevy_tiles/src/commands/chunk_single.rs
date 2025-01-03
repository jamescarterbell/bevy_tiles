use bevy::{
    ecs::{entity::Entity, world::World},
    prelude::{Command, DespawnRecursiveExt},
};

use crate::{
    chunks::ChunkCoord,
    commands::get_chunk,
    maps::{TileDims, TileMap, TileSpacing},
};

use super::{get_or_spawn_chunk, TempRemove};

pub struct SpawnChunk<const N: usize = 2> {
    pub map_id: Entity,
    pub chunk_c: [i32; N],
}

impl<const N: usize> Command for SpawnChunk<N> {
    fn apply(self, world: &mut World) {
        let Some(mut map) = world.temp_remove::<TileMap<N>>(self.map_id) else {
            panic!("No tilemap found!")
        };

        get_or_spawn_chunk::<N>(&mut map, self.chunk_c);
    }
}

pub struct DespawnChunk<const N: usize> {
    pub map_id: Entity,
    pub chunk_c: [i32; N],
}

impl<const N: usize> Command for DespawnChunk<N> {
    fn apply(self, world: &mut World) {
        let Some(mut map) = world.temp_remove::<TileMap<N>>(self.map_id) else {
            panic!("No tilemap found!")
        };

        if let Some(chunk) = get_chunk::<N>(&mut map, self.chunk_c) {
            chunk.try_despawn_recursive();
        }
        map.get_chunks_mut().remove(&ChunkCoord(self.chunk_c));
    }
}
