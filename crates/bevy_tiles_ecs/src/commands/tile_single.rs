use bevy::{
    ecs::{entity::Entity, world::World},
    prelude::Command,
};
use bevy_tiles::{
    commands::{insert_tile, take_tile, TempRemove},
    maps::TileMap,
};

use crate::EntityTile;

pub struct SpawnTile<const N: usize> {
    pub map_id: Entity,
    pub tile_c: [i32; N],
    pub tile_id: EntityTile,
}

impl<const N: usize> Command for SpawnTile<N> {
    fn apply(self, world: &mut World) {
        let replaced = {
            let Some(mut map) = world.temp_remove::<TileMap<N>>(self.map_id) else {
                panic!("No tilemap found!")
            };

            insert_tile::<EntityTile, N>(&mut map, self.tile_c, self.tile_id)
        };

        if let Some(replaced) = replaced {
            world.despawn(*replaced);
        }
    }
}

pub struct DespawnTile<const N: usize> {
    pub map_id: Entity,
    pub tile_c: [i32; N],
}

impl<const N: usize> Command for DespawnTile<N> {
    fn apply(self, world: &mut World) {
        if let Some(id) = {
            let Some(mut map) = world.temp_remove::<TileMap<N>>(self.map_id) else {
                panic!("No tilemap found!")
            };

            take_tile::<EntityTile, N>(&mut map, self.tile_c)
        } {
            world.despawn(*id);
        }
    }
}

pub struct SwapTile<const N: usize> {
    pub map_id: Entity,
    pub tile_c_0: [i32; N],
    pub tile_c_1: [i32; N],
}

impl<const N: usize> Command for SwapTile<N> {
    fn apply(self, world: &mut World) {
        if self.tile_c_0 == self.tile_c_1 {
            return;
        }

        let Some(mut map) = world.temp_remove::<TileMap<N>>(self.map_id) else {
            panic!("No tilemap found!")
        };

        let tile_id_0 = take_tile::<EntityTile, N>(&mut map, self.tile_c_0);

        let tile_id_1 = take_tile::<EntityTile, N>(&mut map, self.tile_c_1);

        let res_0 = tile_id_0.map(|tile_id_0| {
            (
                tile_id_0,
                insert_tile::<EntityTile, N>(&mut map, self.tile_c_1, tile_id_0),
            )
        });

        let res_1 = tile_id_1.map(|tile_id_1| {
            (
                tile_id_1,
                insert_tile::<EntityTile, N>(&mut map, self.tile_c_0, tile_id_1),
            )
        });
    }
}

pub struct MoveTile<const N: usize> {
    pub map_id: Entity,
    pub old_c: [i32; N],
    pub new_c: [i32; N],
}

impl<const N: usize> Command for MoveTile<N> {
    fn apply(self, world: &mut World) {
        let replaced = {
            let Some(mut map) = world.temp_remove::<TileMap<N>>(self.map_id) else {
                panic!("No tilemap found!")
            };

            let Some(id) = take_tile::<EntityTile, N>(&mut map, self.old_c) else {
                println!("Couldn't find the old tile :(");
                return;
            };
            insert_tile::<EntityTile, N>(&mut map, self.new_c, id)
        };

        if let Some(replaced) = replaced {
            world.despawn(*replaced);
        }
    }
}
