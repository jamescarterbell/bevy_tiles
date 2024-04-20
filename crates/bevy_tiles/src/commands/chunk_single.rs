use bevy::ecs::{entity::Entity, system::Command, world::World};

use super::{insert_chunk, take_chunk_despawn_tiles};

pub struct SpawnChunk<const N: usize = 2> {
    pub map_id: Entity,
    pub chunk_c: [i32; N],
    pub chunk_id: Entity,
}

impl<const N: usize> Command for SpawnChunk<N> {
    fn apply(self, world: &mut World) {
        insert_chunk::<N>(world, self.map_id, self.chunk_c, self.chunk_id)
    }
}

pub struct DespawnChunk<const N: usize> {
    pub map_id: Entity,
    pub chunk_c: [i32; N],
}

impl<const N: usize> Command for DespawnChunk<N> {
    fn apply(self, world: &mut World) {
        let tile_id = take_chunk_despawn_tiles::<N>(world, self.map_id, self.chunk_c);
        if let Some(id) = tile_id {
            world.despawn(id);
        }
    }
}
