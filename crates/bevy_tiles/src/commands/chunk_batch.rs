use bevy::ecs::{bundle::Bundle, entity::Entity, system::Command, world::World};

use super::{insert_chunk_batch, take_chunk_batch_despawn_tiles};

pub struct SpawnChunkBatch<F, B, IC, const N: usize = 2>
where
    F: Fn([i32; N]) -> B + Send + 'static,
    B: Bundle + Send + 'static,
    IC: IntoIterator<Item = [i32; N]> + Send + 'static,
{
    pub map_id: Entity,
    pub chunk_cs: IC,
    pub bundle_f: F,
}

impl<F, B, IC, const N: usize> Command for SpawnChunkBatch<F, B, IC, N>
where
    F: Fn([i32; N]) -> B + Send + 'static,
    B: Bundle + Send + 'static,
    IC: IntoIterator<Item = [i32; N]> + Send + 'static,
{
    fn apply(self, world: &mut World) {
        let (chunk_cs, bundles): (Vec<[i32; N]>, Vec<B>) = self
            .chunk_cs
            .into_iter()
            .map(|coord| (coord, (self.bundle_f)(coord)))
            .unzip();

        let chunks = chunk_cs
            .into_iter()
            .zip(world.spawn_batch(bundles))
            .collect::<Vec<([i32; N], Entity)>>();

        insert_chunk_batch::<N>(world, self.map_id, chunks);
    }
}

pub struct DespawnChunkBatch<IC, const N: usize = 2>
where
    IC: IntoIterator<Item = [i32; N]> + Send + 'static,
{
    pub map_id: Entity,
    pub chunk_cs: IC,
}

impl<IC, const N: usize> Command for DespawnChunkBatch<IC, N>
where
    IC: IntoIterator<Item = [i32; N]> + Send + 'static,
{
    fn apply(self, world: &mut World) {
        for (_, tile_id) in take_chunk_batch_despawn_tiles::<N>(world, self.map_id, self.chunk_cs) {
            world.despawn(tile_id);
        }
    }
}
