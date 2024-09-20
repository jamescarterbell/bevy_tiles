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
        let Some(mut map) = world.temp_remove::<TileMap<N>>(self.map_id) else {
            panic!("No tilemap found!")
        };

        insert_tile::<B, N>(&mut map, self.tile_c, self.bundle);
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
