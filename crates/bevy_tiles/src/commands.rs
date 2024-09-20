use std::{
    cmp::Eq,
    hash::Hash,
    ops::{Deref, DerefMut},
};

use crate::{
    chunks::{Chunk, ChunkCoord, InMap},
    coords::{calculate_chunk_coordinate, calculate_tile_index},
    maps::TileMap,
    queries::TileBundle,
    tiles::{InChunk, TileCoord, TileIndex},
};

use bevy::{
    ecs::system::EntityCommands,
    prelude::{Bundle, Commands, Component, Entity, EntityWorldMut, World},
    utils::{hashbrown::hash_map::Entry, HashMap},
};

// mod chunk_batch;
// mod chunk_single;
// mod map;
// mod tile_batch;
mod tile_single;

// use chunk_batch::*;
// use chunk_single::*;
// use map::*;
// use tile_batch::*;
use tile_single::*;

/// Applies commands to a specific tile map.
pub struct TileMapCommands<'a, 'w, 's, const N: usize> {
    commands: &'a mut Commands<'w, 's>,
    map_id: Entity,
}

impl<'a, 'w, 's, const N: usize> TileMapCommands<'a, 'w, 's, N> {
    /// Spawns a tile and returns a handle to the underlying entity.
    /// This will despawn any tile that already exists in this coordinate
    pub fn insert_tile<B: TileBundle>(&mut self, tile_c: impl Into<[i32; N]>, bundle: B) {
        let tile_c = tile_c.into();
        self.commands.spawn_tile(self.map_id, tile_c, bundle);
    }

    // /// Spawns tiles from the given iterator using the given function.
    // /// This will despawn any tile that already exists in this coordinate
    // pub fn spawn_tile_batch<F, B, IC>(&mut self, tile_cs: IC, bundle_f: F) -> &mut Self
    // where
    //     F: Fn([i32; N]) -> B + Send + 'static,
    //     B: Bundle + Send + 'static,
    //     IC: IntoIterator<Item = [i32; N]> + Send + 'static,
    // {
    //     self.commands
    //         .spawn_tile_batch(self.map_id, tile_cs, bundle_f);
    //     self
    // }

    /// Despawns a tile.
    pub fn remove_tile<B: TileBundle>(&mut self, tile_c: impl Into<[i32; N]>) -> &mut Self {
        let tile_c = tile_c.into();
        self.commands.remove_tile::<B>(self.map_id, tile_c);
        self
    }

    // /// Despawns tiles from the given iterator.
    // pub fn despawn_tile_batch<IC>(&mut self, tile_cs: IC) -> &mut Self
    // where
    //     IC: IntoIterator<Item = [i32; N]> + Send + 'static,
    // {
    //     self.commands.despawn_tile_batch(self.map_id, tile_cs);
    //     self
    // }

    // /// Moves a tile from one coordinate to another, overwriting and despawning any tile in the new coordinate.
    // pub fn move_tile(
    //     &mut self,
    //     old_c: impl Into<[i32; N]>,
    //     new_c: impl Into<[i32; N]>,
    // ) -> &mut Self {
    //     let old_c = old_c.into();
    //     let new_c = new_c.into();
    //     self.commands.move_tile(self.map_id, old_c, new_c);
    //     self
    // }

    // /// Move tiles from the first coordinate to the second coordinate, despawning
    // /// any tile found in the second coordinate.
    // pub fn move_tile_batch<IC>(&mut self, tile_cs: IC) -> &mut Self
    // where
    //     IC: IntoIterator<Item = ([i32; N], [i32; N])> + Send + 'static,
    // {
    //     self.commands.move_tile_batch(self.map_id, tile_cs);
    //     self
    // }

    // /// Swaps two tiles if both exist, or moves one tile if the other doesn't exist.
    // pub fn swap_tiles(
    //     &mut self,
    //     tile_c_1: impl Into<[i32; N]>,
    //     tile_c_2: impl Into<[i32; N]>,
    // ) -> &mut Self {
    //     let tile_c_1 = tile_c_1.into();
    //     let tile_c_2 = tile_c_2.into();
    //     self.commands.swap_tiles(self.map_id, tile_c_1, tile_c_2);
    //     self
    // }

    // /// Swap tiles from the first coordinate and the second coordinate
    // pub fn swap_tile_batch<IC>(&mut self, tile_cs: IC) -> &mut Self
    // where
    //     IC: IntoIterator<Item = ([i32; N], [i32; N])> + Send + 'static,
    // {
    //     self.commands.swap_tile_batch(self.map_id, tile_cs);
    //     self
    // }

    // /// Manually spawn a chunk entity, note that this will overwrite and despawn existing chunks at this location.
    // pub fn spawn_chunk(
    //     &mut self,
    //     chunk_c: impl Into<[i32; N]>,
    //     bundle: impl Bundle,
    // ) -> EntityCommands<'_> {
    //     let chunk_c = chunk_c.into();
    //     self.commands.spawn_chunk(self.map_id, chunk_c, bundle)
    // }

    // /// Spawns chunks from the given iterator using the given function.
    // /// This will despawn any chunks (and their tiles) that already exists in this coordinate
    // pub fn spawn_chunk_batch_with<F, B, IC>(&mut self, chunk_cs: IC, bundle_f: F) -> &mut Self
    // where
    //     F: Fn([i32; N]) -> B + Send + 'static,
    //     B: Bundle + Send + 'static,
    //     IC: IntoIterator<Item = [i32; N]> + Send + 'static,
    // {
    //     self.commands
    //         .spawn_chunk_batch_with(self.map_id, chunk_cs, bundle_f);
    //     self
    // }

    // /// Recursively despawn a chunk and all it's tiles.
    // pub fn despawn_chunk(&mut self, chunk_c: impl Into<[i32; N]>) -> &mut Self {
    //     let chunk_c = chunk_c.into();
    //     self.commands.despawn_chunk(self.map_id, chunk_c);
    //     self
    // }

    // /// Despawns chunks (and their tiles) from the given iterator.
    // pub fn despawn_chunk_batch<IC>(&mut self, chunk_cs: IC) -> &mut Self
    // where
    //     IC: IntoIterator<Item = [i32; N]> + Send + 'static,
    // {
    //     self.commands.despawn_chunk_batch(self.map_id, chunk_cs);
    //     self
    // }

    // /// Recursively despawns a map and all it's chunks and tiles.
    // pub fn despawn_map(self) {
    //     TileCommandExt::<N>::despawn_map(self.commands, self.map_id);
    // }

    // /// Get the id of the map.
    // pub fn id(&self) -> Entity {
    //     self.map_id
    // }

    // /// Adds entities to the tilemap.
    // pub fn insert(&mut self, bundle: impl Bundle) -> &mut Self {
    //     self.commands.entity(self.map_id).insert(bundle);
    //     self
    // }
}

/// Helper method for creating map specific commands.
pub trait TileCommandExt<'w, 's, const N: usize> {
    /// Gets [TileMapCommands] to apply commands at the tile map level.
    fn tile_map<'a>(&'a mut self, map_id: Entity) -> TileMapCommands<'a, 'w, 's, N>;

    /// Spawns a tile and returns a handle to the underlying entity.
    /// This will despawn any tile that already exists in this coordinate
    fn spawn_tile<B: TileBundle>(&mut self, map_id: Entity, tile_c: [i32; N], bundle: B);

    // /// Spawns tiles from the given iterator using the given function.
    // /// This will despawn any tile that already exists in this coordinate
    // fn spawn_tile_batch<F, B, IC>(&mut self, map_id: Entity, tile_cs: IC, bundle_f: F)
    // where
    //     F: Fn([i32; N]) -> B + Send + 'static,
    //     B: Bundle + Send + 'static,
    //     IC: IntoIterator<Item = [i32; N]> + Send + 'static;

    /// Despawns a tile.
    fn remove_tile<B: TileBundle>(&mut self, map_id: Entity, tile_c: [i32; N]) -> &mut Self;

    // /// Despawns tiles from the given iterator.
    // fn despawn_tile_batch<IC>(&mut self, map_id: Entity, tile_cs: IC)
    // where
    //     IC: IntoIterator<Item = [i32; N]> + Send + 'static;

    // /// Moves a tile from one coordinate to another, overwriting and despawning any tile in the new coordinate.
    // fn move_tile(
    //     &mut self,
    //     map_id: Entity,
    //     old_c: impl Into<[i32; N]>,
    //     new_c: impl Into<[i32; N]>,
    // ) -> &mut Self;

    // /// Move tiles from the first coordinate to the second coordinate, despawning
    // /// any tile found in the second coordinate.
    // fn move_tile_batch<IC>(&mut self, map_id: Entity, tile_cs: IC)
    // where
    //     IC: IntoIterator<Item = ([i32; N], [i32; N])> + Send + 'static;

    // /// Swaps two tiles if both exist, or moves one tile if the other doesn't exist.
    // fn swap_tiles(&mut self, map_id: Entity, tile_c_1: [i32; N], tile_c_2: [i32; N]) -> &mut Self;

    // /// Swap tiles from the first coordinate and the second coordinate
    // fn swap_tile_batch<IC>(&mut self, map_id: Entity, tile_cs: IC)
    // where
    //     IC: IntoIterator<Item = ([i32; N], [i32; N])> + Send + 'static;

    // /// Manually spawn a chunk entity, note that this will overwrite and despawn existing chunks at this location.
    // fn spawn_chunk(
    //     &mut self,
    //     map_id: Entity,
    //     chunk_c: [i32; N],
    //     bundle: impl Bundle,
    // ) -> EntityCommands<'_>;

    // /// Spawns chunks from the given iterator using the given function.
    // /// This will despawn any chunks (and their tiles) that already exists in this coordinate
    // fn spawn_chunk_batch_with<F, B, IC>(&mut self, map_id: Entity, chunk_cs: IC, bundle_f: F)
    // where
    //     F: Fn([i32; N]) -> B + Send + 'static,
    //     B: Bundle + Send + 'static,
    //     IC: IntoIterator<Item = [i32; N]> + Send + 'static;

    // /// Recursively despawn a chunk and all it's tiles.
    // fn despawn_chunk(&mut self, map_id: Entity, chunk_c: [i32; N]) -> &mut Self;

    // /// Despawns chunks (and their tiles) from the given iterator.
    // fn despawn_chunk_batch<IC>(&mut self, map_id: Entity, chunk_cs: IC)
    // where
    //     IC: IntoIterator<Item = [i32; N]> + Send + 'static;

    // /// Spawn a new map.
    // fn spawn_map(
    //     &mut self,
    //     chunk_size: usize,
    //     bundle: impl Bundle,
    // ) -> TileMapCommands<'_, 'w, 's, N>;

    // /// Recursively despawns a map and all it's chunks and tiles.
    // fn despawn_map(&mut self, map_id: Entity) -> &mut Self;
}

impl<'w, 's, const N: usize> TileCommandExt<'w, 's, N> for Commands<'w, 's> {
    fn tile_map(&mut self, map_id: Entity) -> TileMapCommands<'_, 'w, 's, N> {
        TileMapCommands {
            commands: self,
            map_id,
        }
    }

    /// Spawns a tile and returns a handle to the underlying entity.
    /// This will despawn any tile that already exists in this coordinate
    fn spawn_tile<B: TileBundle>(&mut self, map_id: Entity, tile_c: [i32; N], bundle: B) {
        self.add(InsertTile::<B, N> {
            map_id,
            tile_c,
            bundle,
        });
    }

    // /// Spawns tiles from the given iterator using the given function.
    // /// This will despawn any tile that already exists in this coordinate
    // fn spawn_tile_batch<F, B, IC>(&mut self, map_id: Entity, tile_cs: IC, bundle_f: F)
    // where
    //     F: Fn([i32; N]) -> B + Send + 'static,
    //     B: Bundle + Send + 'static,
    //     IC: IntoIterator<Item = [i32; N]> + Send + 'static,
    // {
    //     self.add(SpawnTileBatch::<F, B, IC, N> {
    //         map_id,
    //         tile_cs,
    //         bundle_f,
    //     });
    // }

    /// Despawns a tile.
    fn remove_tile<B: TileBundle>(&mut self, map_id: Entity, tile_c: [i32; N]) -> &mut Self {
        self.add(RemoveTile::<B, N> {
            map_id,
            tile_c,
            bundle: Default::default(),
        });
        self
    }

    // /// Despawns tiles from the given iterator.
    // fn despawn_tile_batch<IC>(&mut self, map_id: Entity, tile_cs: IC)
    // where
    //     IC: IntoIterator<Item = [i32; N]> + Send + 'static,
    // {
    //     self.add(DespawnTileBatch::<IC, N> { map_id, tile_cs });
    // }

    // /// Moves a tile from one coordinate to another, overwriting and despawning any tile in the new coordinate.
    // fn move_tile(
    //     &mut self,
    //     map_id: Entity,
    //     old_c: impl Into<[i32; N]>,
    //     new_c: impl Into<[i32; N]>,
    // ) -> &mut Self {
    //     let old_c = old_c.into();
    //     let new_c = new_c.into();
    //     self.add(MoveTile::<N> {
    //         map_id,
    //         old_c,
    //         new_c,
    //     });
    //     self
    // }

    // /// Move tiles from the first coordinate to the second coordinate, despawning
    // /// any tile found in the second coordinate.
    // fn move_tile_batch<IC>(&mut self, map_id: Entity, tile_cs: IC)
    // where
    //     IC: IntoIterator<Item = ([i32; N], [i32; N])> + Send + 'static,
    // {
    //     self.add(MoveTileBatch::<IC, N> { map_id, tile_cs });
    // }

    // /// Swaps two tiles if both exist, or moves one tile if the other doesn't exist.
    // fn swap_tiles(&mut self, map_id: Entity, tile_c_1: [i32; N], tile_c_2: [i32; N]) -> &mut Self {
    //     self.add(SwapTile::<N> {
    //         map_id,
    //         tile_c_1,
    //         tile_c_2,
    //     });
    //     self
    // }

    // /// Swap tiles from the first coordinate and the second coordinate
    // fn swap_tile_batch<IC>(&mut self, map_id: Entity, tile_cs: IC)
    // where
    //     IC: IntoIterator<Item = ([i32; N], [i32; N])> + Send + 'static,
    // {
    //     self.add(SwapTileBatch::<IC, N> { map_id, tile_cs });
    // }

    // /// Manually spawn a chunk entity, note that this will overwrite and despawn existing chunks at this location.
    // fn spawn_chunk(
    //     &mut self,
    //     map_id: Entity,
    //     chunk_c: [i32; N],
    //     bundle: impl Bundle,
    // ) -> EntityCommands<'_> {
    //     let chunk_id = self.spawn(bundle).id();
    //     self.add(SpawnChunk::<N> {
    //         map_id,
    //         chunk_c,
    //         chunk_id,
    //     });
    //     self.entity(chunk_id)
    // }

    // /// Spawns chunks from the given iterator using the given function.
    // /// This will despawn any chunks (and their tiles) that already exists in this coordinate
    // fn spawn_chunk_batch_with<F, B, IC>(&mut self, map_id: Entity, chunk_cs: IC, bundle_f: F)
    // where
    //     F: Fn([i32; N]) -> B + Send + 'static,
    //     B: Bundle + Send + 'static,
    //     IC: IntoIterator<Item = [i32; N]> + Send + 'static,
    // {
    //     self.add(SpawnChunkBatch::<F, B, IC, N> {
    //         map_id,
    //         chunk_cs,
    //         bundle_f,
    //     });
    // }

    // /// Recursively despawn a chunk and all it's tiles.
    // fn despawn_chunk(&mut self, map_id: Entity, chunk_c: [i32; N]) -> &mut Self {
    //     self.add(DespawnChunk::<N> { map_id, chunk_c });
    //     self
    // }

    // /// Despawns chunks (and their tiles) from the given iterator.
    // fn despawn_chunk_batch<IC>(&mut self, map_id: Entity, chunk_cs: IC)
    // where
    //     IC: IntoIterator<Item = [i32; N]> + Send + 'static,
    // {
    //     self.add(DespawnChunkBatch::<IC, N> { map_id, chunk_cs });
    // }

    // /// Spawn a new map.
    // fn spawn_map(
    //     &mut self,
    //     chunk_size: usize,
    //     bundle: impl Bundle,
    // ) -> TileMapCommands<'_, 'w, 's, N> {
    //     let map_id = self.spawn(bundle).id();
    //     self.add(SpawnMap::<N> { map_id, chunk_size });
    //     TileMapCommands {
    //         map_id,
    //         commands: self,
    //     }
    // }

    // /// Recursively despawns a map and all it's chunks and tiles.
    // fn despawn_map(&mut self, map_id: Entity) -> &mut Self {
    //     self.add(DespawnMap::<N> { map_id });
    //     self
    // }
}

/// Spawns a chunk in the world if needed, inserts the info into the map, and returns
/// and id for reinsertion
#[inline]
fn get_or_spawn_chunk<'a, const N: usize>(
    map: &'a mut TempRemoved<'_, TileMap<N>>,
    chunk_c: [i32; N],
) -> EntityWorldMut<'a> {
    let chunk_id = map
        .get_chunks()
        .get::<ChunkCoord<N>>(&ChunkCoord(chunk_c))
        .cloned();

    if let Some(chunk_id) = chunk_id {
        // Todo: Change this when NLL is fixed :)
        if map.world.entities().contains(chunk_id) {
            return map.world.get_entity_mut(chunk_id).unwrap();
        }
    }

    spawn_chunk(map, chunk_c)
}

#[inline]
fn spawn_chunk<'a, const N: usize>(
    map: &'a mut TempRemoved<'_, TileMap<N>>,
    chunk_c: [i32; N],
) -> EntityWorldMut<'a> {
    let chunk_c = ChunkCoord(chunk_c);
    let chunk_id = map
        .world
        .spawn((Chunk, ChunkCoord(chunk_c.0), InMap(map.source)))
        .id();
    map.get_chunks_mut().insert(chunk_c, chunk_id);
    map.world.get_entity_mut(chunk_id).unwrap()
}

/// Inserts a tile into the given map.
#[inline]
pub fn insert_tile<B: TileBundle, const N: usize>(
    map: &mut TempRemoved<'_, TileMap<N>>,
    tile_c: [i32; N],
    tile_bundle: B,
) {
    let chunk_size = map.get_chunk_size();

    // Take the chunk out and get the id to reinsert it
    let chunk_c = calculate_chunk_coordinate(tile_c, chunk_size);
    let chunk = get_or_spawn_chunk::<N>(map, chunk_c);

    // Insert the tile
    let tile_i = calculate_tile_index(tile_c, chunk_size);

    tile_bundle.insert(chunk, tile_i)
}

// /// Removes a tile from the given map.
// pub fn take_tile_from_map<const N: usize>(
//     world: &mut World,
//     map: &mut TileMap<N>,
//     tile_c: [i32; N],
// ) -> Option<Entity> {
//     let chunk_size = map.get_chunk_size();
//     // Get the old chunk or return
//     let chunk_c = calculate_chunk_coordinate(tile_c, chunk_size);
//     let (chunk_id, mut chunk) = get_chunk::<N>(world, map, chunk_c)?;

//     // Remove the old entity or return if the old entity is already deleted
//     let tile_i = calculate_tile_index(tile_c, chunk_size);

//     let tile = if let Some(mut tile_e) = chunk
//         .0
//         .get_mut(tile_i)
//         .and_then(|tile| tile.take())
//         .and_then(|tile_id| world.get_entity_mut(tile_id))
//     {
//         tile_e.remove::<(TileIndex, TileCoord<2>, InChunk)>();
//         let tile_id = tile_e.id();
//         Some(tile_id)
//     } else {
//         None
//     };

//     world.get_entity_mut(chunk_id).unwrap().insert(chunk);
//     tile
// }

/// Take a tile from the world.
#[inline]
pub fn remove_tile<B, const N: usize>(
    world: &mut World,
    map_id: Entity,
    tile_c: [i32; N],
) -> Option<B>
where
    B: TileBundle,
{
    let map = world.get_mut::<TileMap<N>>(map_id)?;
    let chunk_size = map.get_chunk_size();
    let chunk_c = calculate_chunk_coordinate(tile_c, chunk_size);
    let chunk_id = *map.get_chunks().get(&ChunkCoord::<N>(chunk_c))?;

    let chunk_e = world.get_entity_mut(chunk_id)?;

    B::remove(chunk_e, calculate_tile_index(tile_c, chunk_size))
}

// /// Inserts a list of entities into the corresponding tiles of a given tile map
// pub fn insert_tile_batch<const N: usize>(
//     world: &mut World,
//     map_id: Entity,
//     tiles: impl IntoIterator<Item = ([i32; N], Entity)>,
// ) {
//     // Remove the map, or spawn an entity to hold the map, then create an empty map
//     let mut map = remove_map::<N>(world, map_id)
//         .unwrap_or_else(|| panic!("Could not find tile map with id '{:?}'", map_id));

//     let chunk_size = map.get_chunk_size();

//     let chunked_tiles = tiles
//         .into_iter()
//         .group_by(|(tile_c, _)| calculate_chunk_coordinate(*tile_c, chunk_size));

//     // Get the chunks and entities from the map
//     let tiles_with_chunk = Vec::from_iter(chunked_tiles.into_iter().map(|(chunk_c, tiles)| {
//         let (chunk_id, chunk) = get_or_spawn_chunk::<N>(world, &mut map, map_id, chunk_c);
//         (chunk_id, chunk, tiles)
//     }));

//     for (chunk_id, mut chunk, tiles) in tiles_with_chunk {
//         for (tile_c, tile_id) in tiles {
//             let tile_i = calculate_tile_index(tile_c, chunk_size);

//             if let Some(tile) = chunk.0.get_mut(tile_i) {
//                 if let Some(old_tile_id) = tile.replace(tile_id) {
//                     world.despawn(old_tile_id);
//                 }
//             }

//             world.get_entity_mut(tile_id).unwrap().insert((
//                 TileIndex(tile_i),
//                 TileCoord::<N>(tile_c),
//                 InChunk(chunk_id),
//             ));
//         }

//         world.get_entity_mut(chunk_id).unwrap().insert(chunk);
//     }

//     world.get_entity_mut(map_id).unwrap().insert(map);
// }

// /// Removes the tiles from the tile map, returning the tile coordinates removed and their corresponding entities.
// pub fn take_tile_batch<const N: usize>(
//     world: &mut World,
//     map_id: Entity,
//     tiles: impl IntoIterator<Item = [i32; N]>,
// ) -> Vec<([i32; N], Entity)> {
//     // Remove the map, or return if it doesn't exist
//     let Some(mut map) = remove_map::<N>(world, map_id) else {
//         return Vec::new();
//     };

//     let chunk_size = map.get_chunk_size();

//     // Group tiles by chunk
//     let chunked_tiles = tiles
//         .into_iter()
//         .group_by(|tile_c| calculate_chunk_coordinate(*tile_c, chunk_size));

//     // Get the chunks and entities from the map
//     let tiles_with_chunk = chunked_tiles
//         .into_iter()
//         .filter_map(|(chunk_c, tiles)| {
//             get_chunk::<N>(world, &mut map, chunk_c)
//                 .map(|chunk_info| (chunk_info.0, chunk_info.1, tiles))
//         })
//         .map(|(chunk_id, chunk, tiles)| {
//             (
//                 chunk_id,
//                 chunk,
//                 tiles.into_iter().collect::<Vec<[i32; N]>>(),
//             )
//         })
//         .collect::<Vec<(Entity, Chunk, Vec<[i32; N]>)>>();

//     let mut tile_ids = Vec::new();
//     for (chunk_id, mut chunk, tiles) in tiles_with_chunk {
//         for tile_c in tiles {
//             let tile_i = calculate_tile_index(tile_c, chunk_size);

//             if let Some(mut tile_e) = chunk
//                 .0
//                 .get_mut(tile_i)
//                 .and_then(|tile| tile.take())
//                 .and_then(|tile_id| world.get_entity_mut(tile_id))
//             {
//                 tile_e.remove::<(TileIndex, TileCoord<N>, InChunk)>();
//                 let tile_id = tile_e.id();
//                 tile_ids.push((tile_c, tile_id));
//             }
//         }

//         world.get_entity_mut(chunk_id).unwrap().insert(chunk);
//     }

//     world.get_entity_mut(map_id).unwrap().insert(map);
//     tile_ids
// }

// /// Insert the given entity into the map and have it treated as a chunk
// pub fn insert_chunk<const N: usize>(
//     world: &mut World,
//     map_id: Entity,
//     chunk_c: [i32; N],
//     chunk_id: Entity,
// ) {
//     let mut map = remove_map::<N>(world, map_id)
//         .unwrap_or_else(|| panic!("Could not find tile map with id '{:?}'", map_id));

//     let chunk_size = map.get_chunk_size();

//     // Despawn the chunk if it exists
//     if let Some(old_chunk) = take_chunk_despawn_tiles_inner::<N>(world, &mut map, chunk_c) {
//         world.despawn(old_chunk);
//     }

//     let chunk_c = ChunkCoord(chunk_c);
//     world.get_entity_mut(chunk_id).unwrap().insert((
//         Chunk::new(chunk_size.pow(N as u32)),
//         ChunkCoord(chunk_c.0),
//         InMap(map_id),
//     ));
//     map.get_chunks_mut().insert(chunk_c, chunk_id);

//     world.entity_mut(map_id).insert(map);
// }

// /// Remove the chunk from the map without despawning it.
// /// # Note
// /// This does not despawn or remove the tile entities, and reinsertion of this entity will not recreate the link to the chunk's tiles.
// /// If you wish to take the chunk and delete it's underlying tiles, use (take_chunk_despawn_tiles)[`take_chunk_despawn_tiles`]
// pub fn take_chunk<const N: usize>(
//     world: &mut World,
//     map_id: Entity,
//     chunk_c: [i32; N],
// ) -> Option<Entity> {
//     // Get the map or return
//     let mut map = remove_map::<N>(world, map_id)?;

//     // Get the old chunk or return
//     let chunk_id = if let Some(mut chunk_e) = map
//         .get_chunks_mut()
//         .remove::<ChunkCoord<N>>(&ChunkCoord(chunk_c))
//         .and_then(|chunk_id| world.get_entity_mut(chunk_id))
//     {
//         let (chunk, _, _) = chunk_e.take::<(Chunk, ChunkCoord<2>, InMap)>().unwrap();
//         let chunk_id = chunk_e.id();
//         for tile_id in chunk.0.into_iter().flatten() {
//             if let Some(mut tile) = world.get_entity_mut(tile_id) {
//                 tile.remove::<InChunk>();
//             }
//         }
//         Some(chunk_id)
//     } else {
//         None
//     };

//     world.entity_mut(map_id).insert(map);

//     chunk_id
// }

// /// Remove the chunk from the map without despawning it and despawns the tiles in the chunk.
// pub fn take_chunk_despawn_tiles<const N: usize>(
//     world: &mut World,
//     map_id: Entity,
//     chunk_c: [i32; N],
// ) -> Option<Entity> {
//     // Get the map or return
//     let mut map = remove_map::<N>(world, map_id)?;

//     let chunk_id = take_chunk_despawn_tiles_inner::<N>(world, &mut map, chunk_c);

//     world.entity_mut(map_id).insert(map);

//     chunk_id
// }

// pub(crate) fn take_chunk_despawn_tiles_inner<const N: usize>(
//     world: &mut World,
//     map: &mut TileMap<N>,
//     chunk_c: [i32; N],
// ) -> Option<Entity> {
//     // Get the old chunk or return
//     let chunk_c = ChunkCoord(chunk_c);
//     if let Some(chunk_id) = map.get_chunks_mut().remove::<ChunkCoord<N>>(&chunk_c) {
//         despawn_chunk_tiles::<N>(world, chunk_id)
//     } else {
//         None
//     }
// }

// pub(crate) fn despawn_chunk_tiles<const N: usize>(
//     world: &mut World,
//     chunk_id: Entity,
// ) -> Option<Entity> {
//     let mut chunk_e = world.get_entity_mut(chunk_id)?;
//     let (chunk, _, _) = chunk_e.take::<(Chunk, ChunkCoord<N>, InMap)>().unwrap();
//     let chunk_id = chunk_e.id();
//     for tile_id in chunk.0.into_iter().flatten() {
//         world.despawn(tile_id);
//     }
//     Some(chunk_id)
// }

// /// Inserts a list of entities into map and treats them as chunks
// pub fn insert_chunk_batch<const N: usize>(
//     world: &mut World,
//     map_id: Entity,
//     chunks: impl IntoIterator<Item = ([i32; N], Entity)>,
// ) {
//     // Remove the map, or spawn an entity to hold the map, then create an empty map
//     let mut map = remove_map::<N>(world, map_id)
//         .unwrap_or_else(|| panic!("Could not find tile map with id '{:?}'", map_id));

//     let chunk_size = map.get_chunk_size();

//     // Get the chunks and entities from the map
//     for (chunk_c, chunk_id) in chunks.into_iter() {
//         if let Some(old_chunk) = take_chunk_despawn_tiles_inner::<N>(world, &mut map, chunk_c) {
//             world.despawn(old_chunk);
//         }

//         let chunk_c = ChunkCoord(chunk_c);
//         world.get_entity_mut(chunk_id).unwrap().insert((
//             Chunk::new(chunk_size.pow(N as u32)),
//             ChunkCoord(chunk_c.0),
//             InMap(map_id),
//         ));
//         map.get_chunks_mut().insert(chunk_c, chunk_id);
//     }

//     world.get_entity_mut(map_id).unwrap().insert(map);
// }

// /// Removes the chunks from the tile map, returning the chunk coordinates removed and their corresponding entities.
// /// # Note
// /// This does not despawn or remove the tile entities, and reinsertion of this entity will not recreate the link to the chunk's tiles.
// /// If you wish to take the chunk and delete it's underlying tiles, use (take_chunk_batch_despawn_tiles)[`take_chunk_batch_despawn_tiles`]
// pub fn take_chunk_batch<const N: usize>(
//     world: &mut World,
//     map_id: Entity,
//     chunks: impl IntoIterator<Item = [i32; N]>,
// ) -> Vec<([i32; N], Entity)> {
//     // Remove the map, or return if it doesn't exist
//     let mut map = if let Some(map_info) = remove_map::<N>(world, map_id) {
//         map_info
//     } else {
//         return Vec::new();
//     };

//     let mut chunk_ids = Vec::new();

//     for chunk_c in chunks.into_iter() {
//         // Get the old chunk or return
//         if let Some(mut chunk_e) = map
//             .get_chunks_mut()
//             .remove::<ChunkCoord<N>>(&ChunkCoord(chunk_c))
//             .and_then(|chunk_id| world.get_entity_mut(chunk_id))
//         {
//             let (chunk, _, _) = chunk_e.take::<(Chunk, ChunkCoord<2>, InMap)>().unwrap();
//             let chunk_id = chunk_e.id();
//             for tile_id in chunk.0.into_iter().flatten() {
//                 if let Some(mut tile) = world.get_entity_mut(tile_id) {
//                     tile.remove::<InChunk>();
//                 }
//             }
//             chunk_ids.push((chunk_c, chunk_id));
//         };
//     }

//     world.get_entity_mut(map_id).unwrap().insert(map);
//     chunk_ids
// }

// /// Removes the chunks from the tile map, returning the chunk coordinates removed and their corresponding entities.
// /// Also despawns all tiles in all the removed chunks.
// pub fn take_chunk_batch_despawn_tiles<const N: usize>(
//     world: &mut World,
//     map_id: Entity,
//     chunks: impl IntoIterator<Item = [i32; N]>,
// ) -> Vec<([i32; N], Entity)> {
//     // Remove the map, or return if it doesn't exist
//     let mut map = if let Some(map_info) = remove_map::<N>(world, map_id) {
//         map_info
//     } else {
//         return Vec::new();
//     };

//     let mut chunk_ids = Vec::new();

//     for chunk_c in chunks.into_iter() {
//         // Get the old chunk or return
//         if let Some(mut chunk_e) = map
//             .get_chunks_mut()
//             .remove::<ChunkCoord<N>>(&ChunkCoord(chunk_c))
//             .and_then(|chunk_id| world.get_entity_mut(chunk_id))
//         {
//             let (chunk, _, _) = chunk_e.take::<(Chunk, ChunkCoord<2>, InMap)>().unwrap();
//             let chunk_id = chunk_e.id();
//             for tile_id in chunk.0.into_iter().flatten() {
//                 world.despawn(tile_id);
//             }
//             chunk_ids.push((chunk_c, chunk_id));
//         };
//     }

//     world.get_entity_mut(map_id).unwrap().insert(map);
//     chunk_ids
// }

// /// Insert the given entity and have it be treated as the given map.
// pub fn insert_map<const N: usize>(world: &mut World, map_id: Entity, chunk_size: usize) {
//     if let Some(mut map) = remove_map::<N>(world, map_id) {
//         despawn_children::<N>(world, &mut map);
//     }
//     world
//         .entity_mut(map_id)
//         .insert(TileMap::<N>::with_chunk_size(chunk_size));
// }

// /// Despawns all the chunks and tiles in a given map
// pub fn despawn_children<const N: usize>(world: &mut World, map: &mut TileMap<N>) {
//     for (_, chunk_id) in map.get_chunks_mut().drain() {
//         if let Some(chunk_id) = despawn_chunk_tiles::<N>(world, chunk_id) {
//             world.despawn(chunk_id);
//         }
//     }
// }

// trait GroupBy: Iterator {
//     fn group_by<F, K>(
//         self,
//         f: F,
//     ) -> bevy::utils::hashbrown::hash_map::IntoIter<
//         K,
//         std::vec::Vec<<Self as std::iter::Iterator>::Item>,
//     >
//     where
//         F: Fn(&Self::Item) -> K,
//         K: Eq + Hash;
// }

// impl<T> GroupBy for T
// where
//     T: Iterator,
// {
//     fn group_by<F, K>(
//         self,
//         f: F,
//     ) -> bevy::utils::hashbrown::hash_map::IntoIter<
//         K,
//         std::vec::Vec<<T as std::iter::Iterator>::Item>,
//     >
//     where
//         F: Fn(&Self::Item) -> K,
//         K: Eq + Hash,
//     {
//         let mut map = HashMap::new();
//         for item in self {
//             let key = f(&item);
//             match map.entry(key) {
//                 Entry::Vacant(v) => {
//                     v.insert(vec![item]);
//                 }
//                 Entry::Occupied(mut o) => o.get_mut().push(item),
//             }
//         }
//         map.into_iter()
//     }
// }

/// Temporarily removed bundle from the world.
pub struct TempRemoved<'w, T: Bundle> {
    value: Option<T>,
    world: &'w mut World,
    source: Entity,
}

impl<'w, T: Bundle> Drop for TempRemoved<'w, T> {
    #[inline]
    fn drop(&mut self) {
        self.world
            .get_entity_mut(self.source)
            .unwrap()
            .insert(self.value.take().unwrap());
    }
}

impl<'w, T: Bundle> Deref for TempRemoved<'w, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.value.as_ref().unwrap()
    }
}

impl<'w, T: Bundle> DerefMut for TempRemoved<'w, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.as_mut().unwrap()
    }
}

/// Temporarily remove a given group of components from an entity
/// and put them back when done using them automatically.
pub trait TempRemove {
    /// Remove components and return a reference to the world and the removed components.
    fn temp_remove<T: Bundle>(&mut self, id: Entity) -> Option<TempRemoved<'_, T>>;
}

impl TempRemove for World {
    #[inline]
    fn temp_remove<T: Bundle>(&mut self, id: Entity) -> Option<TempRemoved<'_, T>> {
        self.get_entity_mut(id)
            .and_then(|mut ent| ent.take::<T>().map(|val| (ent.id(), val)))
            .map(|(id, val)| TempRemoved {
                value: Some(val),
                world: self,
                source: id,
            })
    }
}
