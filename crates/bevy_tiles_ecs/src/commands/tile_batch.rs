use std::iter::repeat;

use bevy::prelude::{Bundle, Command, Entity, World};
use bevy_tiles::{
    commands::{insert_tile_batch, TempRemove},
    maps::TileMap,
};

use crate::EntityTile;

pub struct SpawnTileBatch<TC, TB, const N: usize> {
    pub map_id: Entity,
    pub tile_cs: TC,
    pub tile_b: TB,
}

impl<TC, TB, const N: usize> Command for SpawnTileBatch<TC, TB, N>
where
    TC: Send + IntoIterator<Item = [i32; N]> + 'static,
    TB: Bundle + Clone,
{
    fn apply(self, world: &mut World) {
        let replaced = {
            let Some(mut map) = world.temp_remove::<TileMap<N>>(self.map_id) else {
                panic!("No tilemap found!")
            };

            let mut tile_cs = Vec::new();
            for tile in self.tile_cs {
                tile_cs.push(tile);
            }

            let spawned: Vec<EntityTile> = map
                .get_world_mut()
                .spawn_batch(repeat(self.tile_b).take(tile_cs.len()))
                .map(EntityTile)
                .collect();

            insert_tile_batch::<EntityTile, N>(&mut map, tile_cs, spawned)
        };

        for replaced in replaced {
            world.despawn(*replaced);
        }
    }
}
