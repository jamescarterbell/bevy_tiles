use bevy::{
    ecs::{bundle::Bundle, entity::Entity, system::Command, world::World},
    utils::HashMap,
};
use bimap::BiMap;

use crate::prelude::TileMapLabel;

use super::{insert_tile_batch, take_tile_batch};

pub struct SpawnTileBatch<L, F, B, IC, const N: usize = 2>
where
    L: TileMapLabel + Send + 'static,
    F: Fn([isize; N]) -> B + Send + 'static,
    B: Bundle + Send + 'static,
    IC: IntoIterator<Item = [isize; N]> + Send + 'static,
{
    pub tile_cs: IC,
    pub bundle_f: F,
    pub label: std::marker::PhantomData<L>,
}

impl<L, F, B, IC, const N: usize> Command for SpawnTileBatch<L, F, B, IC, N>
where
    L: TileMapLabel + Send + 'static,
    F: Fn([isize; N]) -> B + Send + 'static,
    B: Bundle + Send + 'static,
    IC: IntoIterator<Item = [isize; N]> + Send + 'static,
{
    fn apply(self, world: &mut World) {
        let (tile_cs, bundles): (Vec<[isize; N]>, Vec<B>) = self
            .tile_cs
            .into_iter()
            .map(|coord| (coord, (self.bundle_f)(coord)))
            .unzip();

        let tiles = tile_cs
            .into_iter()
            .zip(world.spawn_batch(bundles))
            .collect::<Vec<([isize; N], Entity)>>();

        insert_tile_batch::<L, N>(world, tiles);
    }
}

pub struct DespawnTileBatch<L, IC, const N: usize = 2>
where
    L: TileMapLabel + Send + 'static,
    IC: IntoIterator<Item = [isize; N]> + Send + 'static,
{
    pub tile_cs: IC,
    pub label: std::marker::PhantomData<L>,
}

impl<L, IC, const N: usize> Command for DespawnTileBatch<L, IC, N>
where
    L: TileMapLabel + Send + 'static,
    IC: IntoIterator<Item = [isize; N]> + Send + 'static,
{
    fn apply(self, world: &mut World) {
        for (_, tile_id) in take_tile_batch::<L, N>(world, self.tile_cs) {
            world.despawn(tile_id);
        }
    }
}

pub struct MoveTileBatch<L, IC, const N: usize = 2>
where
    L: TileMapLabel + Send + 'static,
    IC: IntoIterator<Item = ([isize; N], [isize; N])> + Send + 'static,
{
    pub tile_cs: IC,
    pub label: std::marker::PhantomData<L>,
}

impl<L, IC, const N: usize> Command for MoveTileBatch<L, IC, N>
where
    L: TileMapLabel + Send + 'static,
    IC: IntoIterator<Item = ([isize; N], [isize; N])> + Send + 'static,
{
    fn apply(self, world: &mut World) {
        const ERR_MESSAGE: &str =
            "Couldn't find tile coord in batch move.  Maybe repeated tile coord in command.";

        let mut tile_cs = self
            .tile_cs
            .into_iter()
            .collect::<HashMap<[isize; N], [isize; N]>>();

        let removed =
            take_tile_batch::<L, N>(world, tile_cs.keys().cloned().collect::<Vec<[isize; N]>>())
                .into_iter()
                .map(|(tile_c, tile_id)| (tile_cs.remove(&tile_c).expect(ERR_MESSAGE), tile_id));

        insert_tile_batch::<L, N>(world, removed);
    }
}

pub struct SwapTileBatch<L, IC, const N: usize = 2>
where
    L: TileMapLabel + Send + 'static,
    IC: IntoIterator<Item = ([isize; N], [isize; N])> + Send + 'static,
{
    pub tile_cs: IC,
    pub label: std::marker::PhantomData<L>,
}

impl<L, IC, const N: usize> Command for SwapTileBatch<L, IC, N>
where
    L: TileMapLabel + Send + 'static,
    IC: IntoIterator<Item = ([isize; N], [isize; N])> + Send + 'static,
{
    fn apply(self, world: &mut World) {
        const ERR_MESSAGE: &str =
            "Couldn't find tile coord in batch move.  Maybe repeated tile coord in command.";

        let tile_cs = self
            .tile_cs
            .into_iter()
            .collect::<BiMap<[isize; N], [isize; N]>>();

        let removed_left = take_tile_batch::<L, N>(
            world,
            tile_cs.left_values().cloned().collect::<Vec<[isize; N]>>(),
        )
        .into_iter()
        .map(|(tile_c, tile_id)| (*tile_cs.get_by_left(&tile_c).expect(ERR_MESSAGE), tile_id));

        let removed_right = take_tile_batch::<L, N>(
            world,
            tile_cs.right_values().cloned().collect::<Vec<[isize; N]>>(),
        )
        .into_iter()
        .map(|(tile_c, tile_id)| (*tile_cs.get_by_right(&tile_c).expect(ERR_MESSAGE), tile_id));

        insert_tile_batch::<L, N>(world, removed_left.chain(removed_right));
    }
}
