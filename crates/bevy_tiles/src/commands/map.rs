use std::ops::Deref;

use bevy::ecs::{entity::Entity, query::With, system::Command, world::World};

use crate::{
    maps::{MapLabel, TileMap},
    prelude::{ChunkCoord, TileMapLabel},
};

use super::{insert_map, take_chunk_despawn_tiles_inner};

pub struct SpawnMap<L, const N: usize = 2> {
    pub map_id: Entity,
    pub label: std::marker::PhantomData<L>,
}

impl<L, const N: usize> Command for SpawnMap<L, N>
where
    L: TileMapLabel + 'static,
{
    fn apply(self, world: &mut World) {
        insert_map::<L, N>(world, self.map_id)
    }
}

pub struct DespawnMap<L, const N: usize = 2> {
    pub label: std::marker::PhantomData<L>,
}

impl<L, const N: usize> Default for DespawnMap<L, N> {
    fn default() -> Self {
        Self {
            label: Default::default(),
        }
    }
}

impl<L, const N: usize> Command for DespawnMap<L, N>
where
    L: TileMapLabel + 'static,
{
    fn apply(self, world: &mut World) {
        if let Ok((map_id)) = world
            .query_filtered::<Entity, (With<MapLabel<L>>, With<TileMap<N>>)>()
            .get_single_mut(world)
        {
            let mut map = world
                .get_entity_mut(map_id)
                .unwrap()
                .take::<TileMap<N>>()
                .unwrap();
            let chunks: Vec<ChunkCoord<N>> = map.chunks.keys().cloned().collect();
            for chunk_c in chunks {
                if let Some(old_chunk) =
                    take_chunk_despawn_tiles_inner::<L, N>(world, *chunk_c, &mut map)
                {
                    world.despawn(old_chunk);
                }
            }
            world.despawn(map_id);
        }
    }
}
