use bevy::ecs::{entity::Entity, system::Command, world::World};

use crate::prelude::TileMapLabel;

use super::{insert_tile, take_tile};

pub struct SpawnTile<L, const N: usize = 2> {
    pub tile_c: [isize; N],
    pub tile_id: Entity,
    pub label: std::marker::PhantomData<L>,
}

impl<L, const N: usize> Command for SpawnTile<L, N>
where
    L: TileMapLabel + Send + 'static,
{
    fn apply(self, world: &mut World) {
        insert_tile::<L, N>(world, self.tile_c, self.tile_id)
    }
}

pub struct DespawnTile<L, const N: usize> {
    pub tile_c: [isize; N],
    pub label: std::marker::PhantomData<L>,
}

impl<L, const N: usize> Command for DespawnTile<L, N>
where
    L: TileMapLabel + Send + 'static,
{
    fn apply(self, world: &mut World) {
        let tile_id = take_tile::<L, N>(world, self.tile_c);
        if let Some(id) = tile_id {
            world.despawn(id);
        }
    }
}

pub struct SwapTile<L, const N: usize> {
    pub tile_c_1: [isize; N],
    pub tile_c_2: [isize; N],
    pub label: std::marker::PhantomData<L>,
}

impl<L, const N: usize> Command for SwapTile<L, N>
where
    L: TileMapLabel + Send + 'static,
{
    fn apply(self, world: &mut World) {
        if self.tile_c_1 == self.tile_c_2 {
            return;
        }

        let tile_id_1 = take_tile::<L, N>(world, self.tile_c_1);

        let tile_id_2 = take_tile::<L, N>(world, self.tile_c_2);

        if let Some(tile_id) = tile_id_1 {
            SpawnTile::<L, N> {
                tile_c: self.tile_c_2,
                tile_id,
                label: self.label,
            }
            .apply(world);
        }

        if let Some(tile_id) = tile_id_2 {
            SpawnTile::<L, N> {
                tile_c: self.tile_c_1,
                tile_id,
                label: self.label,
            }
            .apply(world);
        }
    }
}

pub struct MoveTile<L, const N: usize> {
    pub old_c: [isize; N],
    pub new_c: [isize; N],
    pub label: std::marker::PhantomData<L>,
}

impl<L, const N: usize> Command for MoveTile<L, N>
where
    L: TileMapLabel + Send + 'static,
{
    fn apply(self, world: &mut World) {
        if self.old_c == self.new_c {
            return;
        }

        let old_tile_id = take_tile::<L, N>(world, self.old_c);

        if let Some(old_tile_id) = old_tile_id {
            SpawnTile::<L, N> {
                tile_c: self.new_c,
                tile_id: old_tile_id,
                label: self.label,
            }
            .apply(world);
        }
    }
}
