use bevy::ecs::{entity::Entity, system::Command, world::World};

use super::{insert_tile, insert_tile_into_map, remove_map, take_tile, take_tile_from_map};

pub struct SpawnTile<const N: usize = 2> {
    pub map_id: Entity,
    pub tile_c: [i32; N],
    pub tile_id: Entity,
}

impl<const N: usize> Command for SpawnTile<N> {
    fn apply(self, world: &mut World) {
        insert_tile::<N>(world, self.map_id, self.tile_c, self.tile_id)
    }
}

pub struct DespawnTile<const N: usize> {
    pub map_id: Entity,
    pub tile_c: [i32; N],
}

impl<const N: usize> Command for DespawnTile<N> {
    fn apply(self, world: &mut World) {
        let tile_id = take_tile::<N>(world, self.map_id, self.tile_c);
        if let Some(id) = tile_id {
            world.despawn(id);
        }
    }
}

pub struct SwapTile<const N: usize> {
    pub map_id: Entity,
    pub tile_c_1: [i32; N],
    pub tile_c_2: [i32; N],
}

impl<const N: usize> Command for SwapTile<N> {
    fn apply(self, world: &mut World) {
        if self.tile_c_1 == self.tile_c_2 {
            return;
        }

        let Some(mut map) = remove_map::<N>(world, self.map_id) else {
            return;
        };

        let tile_id_1 = take_tile_from_map::<N>(world, &mut map, self.tile_c_1);

        let tile_id_2 = take_tile_from_map::<N>(world, &mut map, self.tile_c_2);

        if let Some(tile_id) = tile_id_1 {
            insert_tile_into_map(world, &mut map, self.map_id, self.tile_c_2, tile_id);
        }

        if let Some(tile_id) = tile_id_2 {
            insert_tile_into_map(world, &mut map, self.map_id, self.tile_c_1, tile_id);
        }

        world.get_entity_mut(self.map_id).unwrap().insert(map);
    }
}

pub struct MoveTile<const N: usize> {
    pub map_id: Entity,
    pub old_c: [i32; N],
    pub new_c: [i32; N],
}

impl<const N: usize> Command for MoveTile<N> {
    fn apply(self, world: &mut World) {
        if self.old_c == self.new_c {
            return;
        }

        let Some(mut map) = remove_map::<N>(world, self.map_id) else {
            return;
        };

        let tile_id = take_tile_from_map::<N>(world, &mut map, self.old_c);

        if let Some(tile_id) = tile_id {
            insert_tile_into_map(world, &mut map, self.map_id, self.new_c, tile_id);
        }

        world.get_entity_mut(self.map_id).unwrap().insert(map);
    }
}
