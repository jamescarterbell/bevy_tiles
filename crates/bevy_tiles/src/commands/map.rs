use bevy::ecs::{entity::Entity, system::Command, world::World};

use super::{despawn_children, insert_map, remove_map};

pub struct SpawnMap<const N: usize = 2> {
    pub map_id: Entity,
    pub chunk_size: usize,
}

impl<const N: usize> Command for SpawnMap<N> {
    fn apply(self, world: &mut World) {
        insert_map::<N>(world, self.map_id, self.chunk_size)
    }
}

pub struct DespawnMap<const N: usize = 2> {
    pub map_id: Entity,
}

impl<const N: usize> Command for DespawnMap<N> {
    fn apply(self, world: &mut World) {
        let mut map = remove_map::<N>(world, self.map_id).unwrap();
        despawn_children::<N>(world, &mut map);
        world.despawn(self.map_id);
    }
}
