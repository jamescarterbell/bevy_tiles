use std::{cmp::Eq, hash::Hash};

use crate::{
    prelude::{
        calculate_chunk_coordinate, calculate_tile_index, Chunk, ChunkCoord, InMap, TileMap,
    },
    tiles::{InChunk, TileCoord, TileIndex},
};

use bevy::{
    ecs::system::EntityCommands,
    prelude::{Bundle, Commands, Entity, World},
    utils::{hashbrown::hash_map::Entry, HashMap},
};

mod chunk_batch;
mod chunk_single;
mod map;
mod tile_batch;
mod tile_single;

use chunk_batch::*;
use chunk_single::*;
use map::*;
use tile_batch::*;
use tile_single::*;

/// Applies commands to a specific tile map.
pub struct TileMapCommands<'a, 'w, 's, const N: usize> {
    commands: &'a mut Commands<'w, 's>,
    map_id: Entity,
}

impl<'a, 'w, 's, const N: usize> TileMapCommands<'a, 'w, 's, N> {
    /// Spawns a tile and returns a handle to the underlying entity.
    /// This will despawn any tile that already exists in this coordinate
    pub fn spawn_tile(&mut self, tile_c: [isize; N], bundle: impl Bundle) -> EntityCommands<'_> {
        self.commands.spawn_tile(self.map_id, tile_c, bundle)
    }

    /// Spawns tiles from the given iterator using the given function.
    /// This will despawn any tile that already exists in this coordinate
    pub fn spawn_tile_batch<F, B, IC>(&mut self, tile_cs: IC, bundle_f: F) -> &mut Self
    where
        F: Fn([isize; N]) -> B + Send + 'static,
        B: Bundle + Send + 'static,
        IC: IntoIterator<Item = [isize; N]> + Send + 'static,
    {
        self.commands
            .spawn_tile_batch(self.map_id, tile_cs, bundle_f);
        self
    }

    /// Despawns a tile.
    pub fn despawn_tile(&mut self, tile_c: [isize; N]) -> &mut Self {
        self.commands.despawn_tile(self.map_id, tile_c);
        self
    }

    /// Despawns tiles from the given iterator.
    pub fn despawn_tile_batch<IC>(&mut self, tile_cs: IC) -> &mut Self
    where
        IC: IntoIterator<Item = [isize; N]> + Send + 'static,
    {
        self.commands.despawn_tile_batch(self.map_id, tile_cs);
        self
    }

    /// Moves a tile from one coordinate to another, overwriting and despawning any tile in the new coordinate.
    pub fn move_tile(&mut self, old_c: [isize; N], new_c: [isize; N]) -> &mut Self {
        self.commands.move_tile(self.map_id, old_c, new_c);
        self
    }

    /// Move tiles from the first coordinate to the second coordinate, despawning
    /// any tile found in the second coordinate.
    pub fn move_tile_batch<IC>(&mut self, tile_cs: IC) -> &mut Self
    where
        IC: IntoIterator<Item = ([isize; N], [isize; N])> + Send + 'static,
    {
        self.commands.move_tile_batch(self.map_id, tile_cs);
        self
    }

    /// Swaps two tiles if both exist, or moves one tile if the other doesn't exist.
    pub fn swap_tiles(&mut self, tile_c_1: [isize; N], tile_c_2: [isize; N]) -> &mut Self {
        self.commands.swap_tiles(self.map_id, tile_c_1, tile_c_2);
        self
    }

    /// Swap tiles from the first coordinate and the second coordinate
    pub fn swap_tile_batch<IC>(&mut self, tile_cs: IC) -> &mut Self
    where
        IC: IntoIterator<Item = ([isize; N], [isize; N])> + Send + 'static,
    {
        self.commands.swap_tile_batch(self.map_id, tile_cs);
        self
    }

    /// Manually spawn a chunk entity, note that this will overwrite and despawn existing chunks at this location.
    pub fn spawn_chunk(&mut self, chunk_c: [isize; N], bundle: impl Bundle) -> EntityCommands<'_> {
        self.commands.spawn_chunk(self.map_id, chunk_c, bundle)
    }

    /// Spawns chunks from the given iterator using the given function.
    /// This will despawn any chunks (and their tiles) that already exists in this coordinate
    pub fn spawn_chunk_batch_with<F, B, IC>(&mut self, chunk_cs: IC, bundle_f: F) -> &mut Self
    where
        F: Fn([isize; N]) -> B + Send + 'static,
        B: Bundle + Send + 'static,
        IC: IntoIterator<Item = [isize; N]> + Send + 'static,
    {
        self.commands
            .spawn_chunk_batch_with(self.map_id, chunk_cs, bundle_f);
        self
    }

    /// Recursively despawn a chunk and all it's tiles.
    pub fn despawn_chunk(&mut self, chunk_c: [isize; N]) -> &mut Self {
        self.commands.despawn_chunk(self.map_id, chunk_c);
        self
    }

    /// Despawns chunks (and their tiles) from the given iterator.
    pub fn despawn_chunk_batch<IC>(&mut self, chunk_cs: IC) -> &mut Self
    where
        IC: IntoIterator<Item = [isize; N]> + Send + 'static,
    {
        self.commands.despawn_chunk_batch(self.map_id, chunk_cs);
        self
    }

    /// Recursively despawns a map and all it's chunks and tiles.
    pub fn despawn_map(self) {
        self.commands.despawn_map::<N>(self.map_id);
    }

    /// Get the id of the map.
    pub fn id(&self) -> Entity {
        self.map_id
    }

    /// Adds entities to the tilemap.
    pub fn insert(&mut self, bundle: impl Bundle) -> &mut Self {
        self.commands.entity(self.map_id).insert(bundle);
        self
    }
}

/// Helper method for creating map specific commands.
pub trait TileCommandExt<'w, 's> {
    /// Gets [TileMapCommands] to apply commands at the tile map level.
    fn tile_map<'a, const N: usize>(&'a mut self, map_id: Entity)
        -> TileMapCommands<'a, 'w, 's, N>;

    /// Spawns a tile and returns a handle to the underlying entity.
    /// This will despawn any tile that already exists in this coordinate
    fn spawn_tile<const N: usize>(
        &mut self,
        map_id: Entity,
        tile_c: [isize; N],
        bundle: impl Bundle,
    ) -> EntityCommands<'_>;

    /// Spawns tiles from the given iterator using the given function.
    /// This will despawn any tile that already exists in this coordinate
    fn spawn_tile_batch<F, B, IC, const N: usize>(
        &mut self,
        map_id: Entity,
        tile_cs: IC,
        bundle_f: F,
    ) where
        F: Fn([isize; N]) -> B + Send + 'static,
        B: Bundle + Send + 'static,
        IC: IntoIterator<Item = [isize; N]> + Send + 'static;

    /// Despawns a tile.
    fn despawn_tile<const N: usize>(&mut self, map_id: Entity, tile_c: [isize; N]) -> &mut Self;

    /// Despawns tiles from the given iterator.
    fn despawn_tile_batch<IC, const N: usize>(&mut self, map_id: Entity, tile_cs: IC)
    where
        IC: IntoIterator<Item = [isize; N]> + Send + 'static;

    /// Moves a tile from one coordinate to another, overwriting and despawning any tile in the new coordinate.
    fn move_tile<const N: usize>(
        &mut self,
        map_id: Entity,
        old_c: [isize; N],
        new_c: [isize; N],
    ) -> &mut Self;

    /// Move tiles from the first coordinate to the second coordinate, despawning
    /// any tile found in the second coordinate.
    fn move_tile_batch<IC, const N: usize>(&mut self, map_id: Entity, tile_cs: IC)
    where
        IC: IntoIterator<Item = ([isize; N], [isize; N])> + Send + 'static;

    /// Swaps two tiles if both exist, or moves one tile if the other doesn't exist.
    fn swap_tiles<const N: usize>(
        &mut self,
        map_id: Entity,
        tile_c_1: [isize; N],
        tile_c_2: [isize; N],
    ) -> &mut Self;

    /// Swap tiles from the first coordinate and the second coordinate
    fn swap_tile_batch<IC, const N: usize>(&mut self, map_id: Entity, tile_cs: IC)
    where
        IC: IntoIterator<Item = ([isize; N], [isize; N])> + Send + 'static;

    /// Manually spawn a chunk entity, note that this will overwrite and despawn existing chunks at this location.
    fn spawn_chunk<const N: usize>(
        &mut self,
        map_id: Entity,
        chunk_c: [isize; N],
        bundle: impl Bundle,
    ) -> EntityCommands<'_>;

    /// Spawns chunks from the given iterator using the given function.
    /// This will despawn any chunks (and their tiles) that already exists in this coordinate
    fn spawn_chunk_batch_with<F, B, IC, const N: usize>(
        &mut self,
        map_id: Entity,
        chunk_cs: IC,
        bundle_f: F,
    ) where
        F: Fn([isize; N]) -> B + Send + 'static,
        B: Bundle + Send + 'static,
        IC: IntoIterator<Item = [isize; N]> + Send + 'static;

    /// Recursively despawn a chunk and all it's tiles.
    fn despawn_chunk<const N: usize>(&mut self, map_id: Entity, chunk_c: [isize; N]) -> &mut Self;

    /// Despawns chunks (and their tiles) from the given iterator.
    fn despawn_chunk_batch<IC, const N: usize>(&mut self, map_id: Entity, chunk_cs: IC)
    where
        IC: IntoIterator<Item = [isize; N]> + Send + 'static;

    /// Spawn a new map.
    fn spawn_map<const N: usize>(
        &mut self,
        chunk_size: usize,
        bundle: impl Bundle,
    ) -> TileMapCommands<'_, 'w, 's, N>;

    /// Recursively despawns a map and all it's chunks and tiles.
    fn despawn_map<const N: usize>(&mut self, map_id: Entity) -> &mut Self;
}

impl<'w, 's> TileCommandExt<'w, 's> for Commands<'w, 's> {
    fn tile_map<const N: usize>(&mut self, map_id: Entity) -> TileMapCommands<'_, 'w, 's, N> {
        TileMapCommands {
            commands: self,
            map_id,
        }
    }

    /// Spawns a tile and returns a handle to the underlying entity.
    /// This will despawn any tile that already exists in this coordinate
    fn spawn_tile<const N: usize>(
        &mut self,
        map_id: Entity,
        tile_c: [isize; N],
        bundle: impl Bundle,
    ) -> EntityCommands<'_> {
        let tile_id = self.spawn(bundle).id();
        self.add(SpawnTile::<N> {
            map_id,
            tile_c,
            tile_id,
        });
        self.entity(tile_id)
    }

    /// Spawns tiles from the given iterator using the given function.
    /// This will despawn any tile that already exists in this coordinate
    fn spawn_tile_batch<F, B, IC, const N: usize>(
        &mut self,
        map_id: Entity,
        tile_cs: IC,
        bundle_f: F,
    ) where
        F: Fn([isize; N]) -> B + Send + 'static,
        B: Bundle + Send + 'static,
        IC: IntoIterator<Item = [isize; N]> + Send + 'static,
    {
        self.add(SpawnTileBatch::<F, B, IC, N> {
            map_id,
            tile_cs,
            bundle_f,
        });
    }

    /// Despawns a tile.
    fn despawn_tile<const N: usize>(&mut self, map_id: Entity, tile_c: [isize; N]) -> &mut Self {
        self.add(DespawnTile::<N> { map_id, tile_c });
        self
    }

    /// Despawns tiles from the given iterator.
    fn despawn_tile_batch<IC, const N: usize>(&mut self, map_id: Entity, tile_cs: IC)
    where
        IC: IntoIterator<Item = [isize; N]> + Send + 'static,
    {
        self.add(DespawnTileBatch::<IC, N> { map_id, tile_cs });
    }

    /// Moves a tile from one coordinate to another, overwriting and despawning any tile in the new coordinate.
    fn move_tile<const N: usize>(
        &mut self,
        map_id: Entity,
        old_c: [isize; N],
        new_c: [isize; N],
    ) -> &mut Self {
        self.add(MoveTile::<N> {
            map_id,
            old_c,
            new_c,
        });
        self
    }

    /// Move tiles from the first coordinate to the second coordinate, despawning
    /// any tile found in the second coordinate.
    fn move_tile_batch<IC, const N: usize>(&mut self, map_id: Entity, tile_cs: IC)
    where
        IC: IntoIterator<Item = ([isize; N], [isize; N])> + Send + 'static,
    {
        self.add(MoveTileBatch::<IC, N> { map_id, tile_cs });
    }

    /// Swaps two tiles if both exist, or moves one tile if the other doesn't exist.
    fn swap_tiles<const N: usize>(
        &mut self,
        map_id: Entity,
        tile_c_1: [isize; N],
        tile_c_2: [isize; N],
    ) -> &mut Self {
        self.add(SwapTile::<N> {
            map_id,
            tile_c_1,
            tile_c_2,
        });
        self
    }

    /// Swap tiles from the first coordinate and the second coordinate
    fn swap_tile_batch<IC, const N: usize>(&mut self, map_id: Entity, tile_cs: IC)
    where
        IC: IntoIterator<Item = ([isize; N], [isize; N])> + Send + 'static,
    {
        self.add(SwapTileBatch::<IC, N> { map_id, tile_cs });
    }

    /// Manually spawn a chunk entity, note that this will overwrite and despawn existing chunks at this location.
    fn spawn_chunk<const N: usize>(
        &mut self,
        map_id: Entity,
        chunk_c: [isize; N],
        bundle: impl Bundle,
    ) -> EntityCommands<'_> {
        let chunk_id = self.spawn(bundle).id();
        self.add(SpawnChunk::<N> {
            map_id,
            chunk_c,
            chunk_id,
        });
        self.entity(chunk_id)
    }

    /// Spawns chunks from the given iterator using the given function.
    /// This will despawn any chunks (and their tiles) that already exists in this coordinate
    fn spawn_chunk_batch_with<F, B, IC, const N: usize>(
        &mut self,
        map_id: Entity,
        chunk_cs: IC,
        bundle_f: F,
    ) where
        F: Fn([isize; N]) -> B + Send + 'static,
        B: Bundle + Send + 'static,
        IC: IntoIterator<Item = [isize; N]> + Send + 'static,
    {
        self.add(SpawnChunkBatch::<F, B, IC, N> {
            map_id,
            chunk_cs,
            bundle_f,
        });
    }

    /// Recursively despawn a chunk and all it's tiles.
    fn despawn_chunk<const N: usize>(&mut self, map_id: Entity, chunk_c: [isize; N]) -> &mut Self {
        self.add(DespawnChunk::<N> { map_id, chunk_c });
        self
    }

    /// Despawns chunks (and their tiles) from the given iterator.
    fn despawn_chunk_batch<IC, const N: usize>(&mut self, map_id: Entity, chunk_cs: IC)
    where
        IC: IntoIterator<Item = [isize; N]> + Send + 'static,
    {
        self.add(DespawnChunkBatch::<IC, N> { map_id, chunk_cs });
    }

    /// Spawn a new map.
    fn spawn_map<const N: usize>(
        &mut self,
        chunk_size: usize,
        bundle: impl Bundle,
    ) -> TileMapCommands<'_, 'w, 's, N> {
        let map_id = self.spawn(bundle).id();
        self.add(SpawnMap::<N> { map_id, chunk_size });
        TileMapCommands {
            map_id,
            commands: self,
        }
    }

    /// Recursively despawns a map and all it's chunks and tiles.
    fn despawn_map<const N: usize>(&mut self, map_id: Entity) -> &mut Self {
        self.add(DespawnMap::<N> { map_id });
        self
    }
}

/// Spawns a chunk in the world if needed, inserts the info into the map, and returns
/// and id for reinsertion
#[inline]
fn spawn_or_remove_chunk<const N: usize>(
    world: &mut World,
    map: &mut TileMap<N>,
    map_id: Entity,
    chunk_c: [isize; N],
) -> (Entity, Chunk) {
    if let Some(chunk_info) = remove_chunk::<N>(world, map, chunk_c) {
        chunk_info
    } else {
        let chunk_c = ChunkCoord(chunk_c);
        let chunk_id = world.spawn((chunk_c, InMap(map_id))).id();
        map.get_chunks_mut().insert(chunk_c, chunk_id);
        (chunk_id, Chunk::new(map.get_chunk_size().pow(N as u32)))
    }
}

/// Removes a chunk from the world if it exists, and returns the info to reinsert it.
#[inline]
fn remove_chunk<const N: usize>(
    world: &mut World,
    map: &mut TileMap<N>,
    chunk_c: [isize; N],
) -> Option<(Entity, Chunk)> {
    map.get_chunks_mut()
        .get::<ChunkCoord<N>>(&ChunkCoord(chunk_c))
        .cloned()
        .and_then(|chunk_id| world.get_entity_mut(chunk_id))
        .map(|mut chunk_e| (chunk_e.id(), chunk_e.take::<Chunk>().unwrap()))
}

/// Takes the map out of the world if it exists.
#[inline]
fn remove_map<const N: usize>(world: &mut World, map_id: Entity) -> Option<TileMap<N>> {
    world
        .get_entity_mut(map_id)
        .and_then(|mut map_e| map_e.take::<TileMap<N>>())
}

/// Inserts a tile into the given map.
pub fn insert_tile_into_map<const N: usize>(
    world: &mut World,
    map: &mut TileMap<N>,
    map_id: Entity,
    tile_c: [isize; N],
    tile_id: Entity,
) {
    let chunk_size = map.get_chunk_size();

    // Take the chunk out and get the id to reinsert it
    let chunk_c = calculate_chunk_coordinate(tile_c, chunk_size);
    let (chunk_id, mut chunk) = spawn_or_remove_chunk::<N>(world, map, map_id, chunk_c);

    // Insert the tile
    let tile_i = calculate_tile_index(tile_c, chunk_size);

    if let Some(tile) = chunk.0.get_mut(tile_i) {
        if let Some(old_tile_id) = tile.replace(tile_id) {
            world.despawn(old_tile_id);
        }
    }

    world.get_entity_mut(tile_id).unwrap().insert((
        TileIndex(tile_i),
        TileCoord::<N>(tile_c),
        InChunk(chunk_id),
    ));

    world.get_entity_mut(chunk_id).unwrap().insert(chunk);
}

/// Inserts a tile into the world.
pub fn insert_tile<const N: usize>(
    world: &mut World,
    map_id: Entity,
    tile_c: [isize; N],
    tile_id: Entity,
) {
    // Take the map out and get the id to reinsert it
    let mut map = remove_map::<N>(world, map_id)
        .unwrap_or_else(|| panic!("Could not find tile map with id '{:?}'", map_id));
    insert_tile_into_map(world, &mut map, map_id, tile_c, tile_id);
    world.get_entity_mut(map_id).unwrap().insert(map);
}

/// Removes a tile from the given map.
pub fn take_tile_from_map<const N: usize>(
    world: &mut World,
    map: &mut TileMap<N>,
    tile_c: [isize; N],
) -> Option<Entity> {
    let chunk_size = map.get_chunk_size();
    // Get the old chunk or return
    let chunk_c = calculate_chunk_coordinate(tile_c, chunk_size);
    let (chunk_id, mut chunk) = remove_chunk::<N>(world, map, chunk_c)?;

    // Remove the old entity or return if the old entity is already deleted
    let tile_i = calculate_tile_index(tile_c, chunk_size);

    let tile = if let Some(mut tile_e) = chunk
        .0
        .get_mut(tile_i)
        .and_then(|tile| tile.take())
        .and_then(|tile_id| world.get_entity_mut(tile_id))
    {
        tile_e.remove::<(TileIndex, TileCoord, InChunk)>();
        let tile_id = tile_e.id();
        Some(tile_id)
    } else {
        None
    };

    world.get_entity_mut(chunk_id).unwrap().insert(chunk);
    tile
}

/// Take a tile from the world.
pub fn take_tile<const N: usize>(
    world: &mut World,
    map_id: Entity,
    tile_c: [isize; N],
) -> Option<Entity> {
    // Get the map or return
    let mut map = remove_map::<N>(world, map_id)?;
    let tile = take_tile_from_map(world, &mut map, tile_c);
    world.get_entity_mut(map_id).unwrap().insert(map);
    tile
}

/// Inserts a list of entities into the corresponding tiles of a given tile map
pub fn insert_tile_batch<const N: usize>(
    world: &mut World,
    map_id: Entity,
    tiles: impl IntoIterator<Item = ([isize; N], Entity)>,
) {
    // Remove the map, or spawn an entity to hold the map, then create an empty map
    let mut map = remove_map::<N>(world, map_id)
        .unwrap_or_else(|| panic!("Could not find tile map with id '{:?}'", map_id));

    let chunk_size = map.get_chunk_size();

    let chunked_tiles = tiles
        .into_iter()
        .group_by(|(tile_c, _)| calculate_chunk_coordinate(*tile_c, chunk_size));

    // Get the chunks and entities from the map
    let tiles_with_chunk = Vec::from_iter(chunked_tiles.into_iter().map(|(chunk_c, tiles)| {
        let (chunk_id, chunk) = spawn_or_remove_chunk::<N>(world, &mut map, map_id, chunk_c);
        (chunk_id, chunk, tiles)
    }));

    for (chunk_id, mut chunk, tiles) in tiles_with_chunk {
        for (tile_c, tile_id) in tiles {
            let tile_i = calculate_tile_index(tile_c, chunk_size);

            if let Some(tile) = chunk.0.get_mut(tile_i) {
                if let Some(old_tile_id) = tile.replace(tile_id) {
                    world.despawn(old_tile_id);
                }
            }

            world.get_entity_mut(tile_id).unwrap().insert((
                TileIndex(tile_i),
                TileCoord::<N>(tile_c),
                InChunk(chunk_id),
            ));
        }

        world.get_entity_mut(chunk_id).unwrap().insert(chunk);
    }

    world.get_entity_mut(map_id).unwrap().insert(map);
}

/// Removes the tiles from the tile map, returning the tile coordinates removed and their corresponding entities.
pub fn take_tile_batch<const N: usize>(
    world: &mut World,
    map_id: Entity,
    tiles: impl IntoIterator<Item = [isize; N]>,
) -> Vec<([isize; N], Entity)> {
    // Remove the map, or return if it doesn't exist
    let Some(mut map) = remove_map::<N>(world, map_id) else {
        return Vec::new();
    };

    let chunk_size = map.get_chunk_size();

    // Group tiles by chunk
    let chunked_tiles = tiles
        .into_iter()
        .group_by(|tile_c| calculate_chunk_coordinate(*tile_c, chunk_size));

    // Get the chunks and entities from the map
    let tiles_with_chunk = chunked_tiles
        .into_iter()
        .filter_map(|(chunk_c, tiles)| {
            remove_chunk::<N>(world, &mut map, chunk_c)
                .map(|chunk_info| (chunk_info.0, chunk_info.1, tiles))
        })
        .map(|(chunk_id, chunk, tiles)| {
            (
                chunk_id,
                chunk,
                tiles.into_iter().collect::<Vec<[isize; N]>>(),
            )
        })
        .collect::<Vec<(Entity, Chunk, Vec<[isize; N]>)>>();

    let mut tile_ids = Vec::new();
    for (chunk_id, mut chunk, tiles) in tiles_with_chunk {
        for tile_c in tiles {
            let tile_i = calculate_tile_index(tile_c, chunk_size);

            if let Some(mut tile_e) = chunk
                .0
                .get_mut(tile_i)
                .and_then(|tile| tile.take())
                .and_then(|tile_id| world.get_entity_mut(tile_id))
            {
                tile_e.remove::<(TileIndex, TileCoord, InChunk)>();
                let tile_id = tile_e.id();
                tile_ids.push((tile_c, tile_id));
            }
        }

        world.get_entity_mut(chunk_id).unwrap().insert(chunk);
    }

    world.get_entity_mut(map_id).unwrap().insert(map);
    tile_ids
}

/// Insert the given entity into the map and have it treated as a chunk
pub fn insert_chunk<const N: usize>(
    world: &mut World,
    map_id: Entity,
    chunk_c: [isize; N],
    chunk_id: Entity,
) {
    let mut map = remove_map::<N>(world, map_id)
        .unwrap_or_else(|| panic!("Could not find tile map with id '{:?}'", map_id));

    let chunk_size = map.get_chunk_size();

    // Despawn the chunk if it exists
    if let Some(old_chunk) = take_chunk_despawn_tiles_inner::<N>(world, &mut map, chunk_c) {
        world.despawn(old_chunk);
    }

    let chunk_c = ChunkCoord(chunk_c);
    world.get_entity_mut(chunk_id).unwrap().insert((
        Chunk::new(chunk_size.pow(N as u32)),
        chunk_c,
        InMap(map_id),
    ));
    map.get_chunks_mut().insert(chunk_c, chunk_id);

    world.entity_mut(map_id).insert(map);
}

/// Remove the chunk from the map without despawning it.
/// # Note
/// This does not despawn or remove the tile entities, and reinsertion of this entity will not recreate the link to the chunk's tiles.
/// If you wish to take the chunk and delete it's underlying tiles, use (take_chunk_despawn_tiles)[`take_chunk_despawn_tiles`]
pub fn take_chunk<const N: usize>(
    world: &mut World,
    map_id: Entity,
    chunk_c: [isize; N],
) -> Option<Entity> {
    // Get the map or return
    let mut map = remove_map::<N>(world, map_id)?;

    // Get the old chunk or return
    let chunk_id = if let Some(mut chunk_e) = map
        .get_chunks_mut()
        .remove::<ChunkCoord<N>>(&ChunkCoord(chunk_c))
        .and_then(|chunk_id| world.get_entity_mut(chunk_id))
    {
        let (chunk, _, _) = chunk_e.take::<(Chunk, ChunkCoord, InMap)>().unwrap();
        let chunk_id = chunk_e.id();
        for tile_id in chunk.0.into_iter().flatten() {
            if let Some(mut tile) = world.get_entity_mut(tile_id) {
                tile.remove::<InChunk>();
            }
        }
        Some(chunk_id)
    } else {
        None
    };

    world.entity_mut(map_id).insert(map);

    chunk_id
}

/// Remove the chunk from the map without despawning it and despawns the tiles in the chunk.
pub fn take_chunk_despawn_tiles<const N: usize>(
    world: &mut World,
    map_id: Entity,
    chunk_c: [isize; N],
) -> Option<Entity> {
    // Get the map or return
    let mut map = remove_map::<N>(world, map_id)?;

    let chunk_id = take_chunk_despawn_tiles_inner::<N>(world, &mut map, chunk_c);

    world.entity_mut(map_id).insert(map);

    chunk_id
}

pub(crate) fn take_chunk_despawn_tiles_inner<const N: usize>(
    world: &mut World,
    map: &mut TileMap<N>,
    chunk_c: [isize; N],
) -> Option<Entity> {
    // Get the old chunk or return
    let chunk_c = ChunkCoord(chunk_c);
    let chunk_id = if let Some(mut chunk_e) = map
        .get_chunks_mut()
        .remove::<ChunkCoord<N>>(&chunk_c)
        .and_then(|chunk_id| world.get_entity_mut(chunk_id))
    {
        let (chunk, _, _) = chunk_e.take::<(Chunk, ChunkCoord, InMap)>().unwrap();
        let chunk_id = chunk_e.id();
        for tile_id in chunk.0.into_iter().flatten() {
            world.despawn(tile_id);
        }
        Some(chunk_id)
    } else {
        None
    };
    chunk_id
}

/// Inserts a list of entities into map and treats them as chunks
pub fn insert_chunk_batch<const N: usize>(
    world: &mut World,
    map_id: Entity,
    chunks: impl IntoIterator<Item = ([isize; N], Entity)>,
) {
    // Remove the map, or spawn an entity to hold the map, then create an empty map
    let mut map = remove_map::<N>(world, map_id)
        .unwrap_or_else(|| panic!("Could not find tile map with id '{:?}'", map_id));

    let chunk_size = map.get_chunk_size();

    // Get the chunks and entities from the map
    for (chunk_c, chunk_id) in chunks.into_iter() {
        if let Some(old_chunk) = take_chunk_despawn_tiles_inner::<N>(world, &mut map, chunk_c) {
            world.despawn(old_chunk);
        }

        let chunk_c = ChunkCoord(chunk_c);
        world.get_entity_mut(chunk_id).unwrap().insert((
            Chunk::new(chunk_size.pow(N as u32)),
            chunk_c,
            InMap(map_id),
        ));
        map.get_chunks_mut().insert(chunk_c, chunk_id);
    }

    world.get_entity_mut(map_id).unwrap().insert(map);
}

/// Removes the chunks from the tile map, returning the chunk coordinates removed and their corresponding entities.
/// # Note
/// This does not despawn or remove the tile entities, and reinsertion of this entity will not recreate the link to the chunk's tiles.
/// If you wish to take the chunk and delete it's underlying tiles, use (take_chunk_batch_despawn_tiles)[`take_chunk_batch_despawn_tiles`]
pub fn take_chunk_batch<const N: usize>(
    world: &mut World,
    map_id: Entity,
    chunks: impl IntoIterator<Item = [isize; N]>,
) -> Vec<([isize; N], Entity)> {
    // Remove the map, or return if it doesn't exist
    let mut map = if let Some(map_info) = remove_map::<N>(world, map_id) {
        map_info
    } else {
        return Vec::new();
    };

    let mut chunk_ids = Vec::new();

    for chunk_c in chunks.into_iter() {
        // Get the old chunk or return
        if let Some(mut chunk_e) = map
            .get_chunks_mut()
            .remove::<ChunkCoord<N>>(&ChunkCoord(chunk_c))
            .and_then(|chunk_id| world.get_entity_mut(chunk_id))
        {
            let (chunk, _, _) = chunk_e.take::<(Chunk, ChunkCoord, InMap)>().unwrap();
            let chunk_id = chunk_e.id();
            for tile_id in chunk.0.into_iter().flatten() {
                if let Some(mut tile) = world.get_entity_mut(tile_id) {
                    tile.remove::<InChunk>();
                }
            }
            chunk_ids.push((chunk_c, chunk_id));
        };
    }

    world.get_entity_mut(map_id).unwrap().insert(map);
    chunk_ids
}

/// Removes the chunks from the tile map, returning the chunk coordinates removed and their corresponding entities.
/// Also despawns all tiles in all the removed chunks.
pub fn take_chunk_batch_despawn_tiles<const N: usize>(
    world: &mut World,
    map_id: Entity,
    chunks: impl IntoIterator<Item = [isize; N]>,
) -> Vec<([isize; N], Entity)> {
    // Remove the map, or return if it doesn't exist
    let mut map = if let Some(map_info) = remove_map::<N>(world, map_id) {
        map_info
    } else {
        return Vec::new();
    };

    let mut chunk_ids = Vec::new();

    for chunk_c in chunks.into_iter() {
        // Get the old chunk or return
        if let Some(mut chunk_e) = map
            .get_chunks_mut()
            .remove::<ChunkCoord<N>>(&ChunkCoord(chunk_c))
            .and_then(|chunk_id| world.get_entity_mut(chunk_id))
        {
            let (chunk, _, _) = chunk_e.take::<(Chunk, ChunkCoord, InMap)>().unwrap();
            let chunk_id = chunk_e.id();
            for tile_id in chunk.0.into_iter().flatten() {
                world.despawn(tile_id);
            }
            chunk_ids.push((chunk_c, chunk_id));
        };
    }

    world.get_entity_mut(map_id).unwrap().insert(map);
    chunk_ids
}

/// Insert the given entity and have it be treated as the given map.
pub fn insert_map<const N: usize>(world: &mut World, map_id: Entity, chunk_size: usize) {
    if let Some(mut map) = remove_map::<N>(world, map_id) {
        despawn_children::<N>(world, &mut map);
    }
    world
        .entity_mut(map_id)
        .insert(TileMap::<N>::with_chunk_size(chunk_size));
}

/// Despawns all the chunks and tiles in a given map
pub fn despawn_children<const N: usize>(world: &mut World, map: &mut TileMap<N>) {
    let chunks: Vec<ChunkCoord<N>> = map.get_chunks().keys().cloned().collect();
    for chunk_c in chunks {
        if let Some(old_chunk) = take_chunk_despawn_tiles_inner::<N>(world, map, *chunk_c) {
            world.despawn(old_chunk);
        }
    }
}

trait GroupBy: Iterator {
    fn group_by<F, K>(
        self,
        f: F,
    ) -> bevy::utils::hashbrown::hash_map::IntoIter<
        K,
        std::vec::Vec<<Self as std::iter::Iterator>::Item>,
    >
    where
        F: Fn(&Self::Item) -> K,
        K: Eq + Hash;
}

impl<T> GroupBy for T
where
    T: Iterator,
{
    fn group_by<F, K>(
        self,
        f: F,
    ) -> bevy::utils::hashbrown::hash_map::IntoIter<
        K,
        std::vec::Vec<<T as std::iter::Iterator>::Item>,
    >
    where
        F: Fn(&Self::Item) -> K,
        K: Eq + Hash,
    {
        let mut map = HashMap::new();
        for item in self {
            let key = f(&item);
            match map.entry(key) {
                Entry::Vacant(v) => {
                    v.insert(vec![item]);
                }
                Entry::Occupied(mut o) => o.get_mut().push(item),
            }
        }
        map.into_iter()
    }
}
