use bevy::ecs::{entity::Entity, system::Command, world::World};

use crate::prelude::TileMapLabel;

use super::{insert_chunk, take_chunk_despawn_tiles};

pub struct SpawnChunk<L, const N: usize = 2> {
    pub chunk_c: [isize; N],
    pub chunk_id: Entity,
    pub label: std::marker::PhantomData<L>,
}

impl<L, const N: usize> Command for SpawnChunk<L, N>
where
    L: TileMapLabel + Send + 'static,
{
    fn apply(self, world: &mut World) {
        insert_chunk::<L, N>(world, self.chunk_c, self.chunk_id)
    }
}

pub struct DespawnChunk<L, const N: usize> {
    pub chunk_c: [isize; N],
    pub label: std::marker::PhantomData<L>,
}

impl<L, const N: usize> Command for DespawnChunk<L, N>
where
    L: TileMapLabel + Send + 'static,
{
    fn apply(self, world: &mut World) {
        let tile_id = take_chunk_despawn_tiles::<L, N>(world, self.chunk_c);
        if let Some(id) = tile_id {
            world.despawn(id);
        }
    }
}
