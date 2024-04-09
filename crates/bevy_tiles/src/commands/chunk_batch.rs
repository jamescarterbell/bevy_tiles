use bevy::ecs::{bundle::Bundle, entity::Entity, system::Command, world::World};

use super::{insert_chunk_batch, take_chunk_batch_despawn_tiles};

pub struct SpawnChunkBatch<F, B, IC, const N: usize = 2>
where
    F: Fn([isize; N]) -> B + Send + 'static,
    B: Bundle + Send + 'static,
    IC: IntoIterator<Item = [isize; N]> + Send + 'static,
{
    pub map_id: Entity,
    pub chunk_cs: IC,
    pub bundle_f: F,
}

impl<F, B, IC, const N: usize> Command for SpawnChunkBatch<F, B, IC, N>
where
    F: Fn([isize; N]) -> B + Send + 'static,
    B: Bundle + Send + 'static,
    IC: IntoIterator<Item = [isize; N]> + Send + 'static,
{
    fn apply(self, world: &mut World) {
        let (chunk_cs, bundles): (Vec<[isize; N]>, Vec<B>) = self
            .chunk_cs
            .into_iter()
            .map(|coord| (coord, (self.bundle_f)(coord)))
            .unzip();

        let chunks = chunk_cs
            .into_iter()
            .zip(world.spawn_batch(bundles))
            .collect::<Vec<([isize; N], Entity)>>();

        insert_chunk_batch::<N>(world, self.map_id, chunks);
    }
}

pub struct DespawnChunkBatch<IC, const N: usize = 2>
where
    IC: IntoIterator<Item = [isize; N]> + Send + 'static,
{
    pub map_id: Entity,
    pub chunk_cs: IC,
}

impl<IC, const N: usize> Command for DespawnChunkBatch<IC, N>
where
    IC: IntoIterator<Item = [isize; N]> + Send + 'static,
{
    fn apply(self, world: &mut World) {
        for (_, tile_id) in take_chunk_batch_despawn_tiles::<N>(world, self.map_id, self.chunk_cs) {
            world.despawn(tile_id);
        }
    }
}
