use std::marker::PhantomData;

use bevy::{
    ecs::{entity::Entity, world::World},
    prelude::Command,
};

use crate::{maps::TileMap, queries::TileBundle};

use super::{insert_tile, remove_tile, TempRemove};

pub struct InsertTile<B, const N: usize>
where
    B: TileBundle,
{
    pub map_id: Entity,
    pub tile_c: [i32; N],
    pub bundle: B,
}

impl<B: TileBundle, const N: usize> Command for InsertTile<B, N> {
    fn apply(self, world: &mut World) {
        if let Some(mut map) = world.temp_remove::<TileMap<N>>(self.map_id) {
            insert_tile::<B, N>(&mut map, self.tile_c, self.bundle);
        }

        panic!("No tilemap found!")
    }
}

pub struct RemoveTile<B, const N: usize>
where
    B: TileBundle,
{
    pub map_id: Entity,
    pub tile_c: [i32; N],
    pub bundle: PhantomData<B>,
}

impl<B, const N: usize> Command for RemoveTile<B, N>
where
    B: TileBundle,
{
    fn apply(self, world: &mut World) {
        remove_tile::<B, N>(world, self.map_id, self.tile_c);
    }
}

// pub struct SwapTile<const N: usize> {
//     pub map_id: Entity,
//     pub tile_c_1: [i32; N],
//     pub tile_c_2: [i32; N],
// }

// impl<const N: usize> Command for SwapTile<N> {
//     fn apply(self, world: &mut World) {
//         if self.tile_c_1 == self.tile_c_2 {
//             return;
//         }

//         let Some(mut map) = world.temp_remove::<TileMap<N>>(self.map_id) else {
//             return;
//         };

//         let tile_id_1 = take_tile_from_map::<N>(world, &mut map, self.tile_c_1);

//         let tile_id_2 = take_tile_from_map::<N>(world, &mut map, self.tile_c_2);

//         if let Some(tile_id) = tile_id_1 {
//             insert_tile_into_map(world, &mut map, self.map_id, self.tile_c_2, tile_id);
//         }

//         if let Some(tile_id) = tile_id_2 {
//             insert_tile_into_map(world, &mut map, self.map_id, self.tile_c_1, tile_id);
//         }

//         world.get_entity_mut(self.map_id).unwrap().insert(map);
//     }
// }

// pub struct MoveTile<const N: usize> {
//     pub map_id: Entity,
//     pub old_c: [i32; N],
//     pub new_c: [i32; N],
// }

// impl<const N: usize> Command for MoveTile<N> {
//     fn apply(self, world: &mut World) {
//         if self.old_c == self.new_c {
//             return;
//         }

//         let Some(mut map) = remove_map::<N>(world, self.map_id) else {
//             return;
//         };

//         let tile_id = take_tile_from_map::<N>(world, &mut map, self.old_c);

//         if let Some(tile_id) = tile_id {
//             insert_tile_into_map(world, &mut map, self.map_id, self.new_c, tile_id);
//         }

//         world.get_entity_mut(self.map_id).unwrap().insert(map);
//     }
// }
