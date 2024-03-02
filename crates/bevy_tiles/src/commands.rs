use std::{
    cmp::Eq,
    hash::Hash,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{
    maps::MapLabel,
    prelude::{
        calculate_chunk_coordinate, calculate_tile_index, Chunk, ChunkCoord, InMap, TileMap,
        TileMapLabel,
    },
    tiles::{InChunk, TileCoord, TileIndex},
};

use bevy::{
    ecs::system::{Command, EntityCommands},
    prelude::{Bundle, Commands, Entity, With, World},
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
pub struct TileCommands<'a, 'w, 's, L, const N: usize> {
    commands: &'a mut Commands<'w, 's>,
    phantom: PhantomData<L>,
}

impl<'a, 'w, 's, L, const N: usize> Deref for TileCommands<'a, 'w, 's, L, N>
where
    L: TileMapLabel + 'static,
{
    type Target = Commands<'w, 's>;

    fn deref(&self) -> &Self::Target {
        self.commands
    }
}

impl<'a, 'w, 's, L, const N: usize> DerefMut for TileCommands<'a, 'w, 's, L, N>
where
    L: TileMapLabel + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.commands
    }
}

/// Helper method for creating map specific commands.
pub trait TileCommandExt<'w, 's> {
    /// Gets the [TileCommands] to apply commands at the tile map level.
    fn tiles<'a, L, const N: usize>(&'a mut self) -> TileCommands<'a, 'w, 's, L, N>
    where
        L: TileMapLabel + 'static;
}

impl<'w, 's> TileCommandExt<'w, 's> for Commands<'w, 's> {
    fn tiles<L, const N: usize>(&mut self) -> TileCommands<'_, 'w, 's, L, N>
    where
        L: TileMapLabel + 'static,
    {
        TileCommands {
            commands: self,
            phantom: PhantomData,
        }
    }
}

impl<'a, 'w, 's, L, const N: usize> TileCommands<'a, 'w, 's, L, N>
where
    L: TileMapLabel + 'static,
{
    /// Spawns a tile and returns a handle to the underlying entity.
    /// This will despawn any tile that already exists in this coordinate
    pub fn spawn_tile<T>(&mut self, tile_c: [isize; N], bundle: T) -> EntityCommands<'_>
    where
        T: Bundle + 'static,
    {
        let tile_id = self.spawn(bundle).id();
        self.add(SpawnTile::<L, N> {
            tile_c,
            tile_id,
            label: std::marker::PhantomData,
        });
        self.entity(tile_id)
    }

    /// Spawns tiles from the given iterator using the given function.
    /// This will despawn any tile that already exists in this coordinate
    pub fn spawn_tile_batch<F, B, IC>(&mut self, tile_cs: IC, bundle_f: F)
    where
        F: Fn([isize; N]) -> B + Send + 'static,
        B: Bundle + Send + 'static,
        IC: IntoIterator<Item = [isize; N]> + Send + 'static,
    {
        self.add(SpawnTileBatch::<L, F, B, IC, N> {
            tile_cs,
            bundle_f,
            label: std::marker::PhantomData,
        });
    }

    /// Despawns a tile.
    pub fn despawn_tile(&mut self, tile_c: [isize; N]) -> &mut Self {
        self.add(DespawnTile::<L, N> {
            tile_c,
            label: PhantomData,
        });
        self
    }

    /// Despawns tiles from the given iterator.
    pub fn despawn_tile_batch<IC>(&mut self, tile_cs: IC)
    where
        IC: IntoIterator<Item = [isize; N]> + Send + 'static,
    {
        self.add(DespawnTileBatch::<L, IC, N> {
            tile_cs,
            label: std::marker::PhantomData,
        });
    }

    /// Moves a tile from one coordinate to another, overwriting and despawning any tile in the new coordinate.
    pub fn move_tile(&mut self, old_c: [isize; N], new_c: [isize; N]) -> &mut Self {
        self.add(MoveTile::<L, N> {
            old_c,
            new_c,
            label: PhantomData,
        });
        self
    }

    /// Move tiles from the first coordinate to the second coordinate, despawning
    /// any tile found in the second coordinate.
    pub fn move_tile_batch<IC>(&mut self, tile_cs: IC)
    where
        IC: IntoIterator<Item = ([isize; N], [isize; N])> + Send + 'static,
    {
        self.add(MoveTileBatch::<L, IC, N> {
            tile_cs,
            label: std::marker::PhantomData,
        });
    }

    /// Swaps two tiles if both exist, or just moves one tile if the other doesn't exist.
    pub fn swap_tiles(&mut self, tile_c_1: [isize; N], tile_c_2: [isize; N]) -> &mut Self {
        self.add(SwapTile::<L, N> {
            tile_c_1,
            tile_c_2,
            label: PhantomData,
        });
        self
    }

    /// Swap tiles from the first coordinate and the second coordinate
    pub fn swap_tile_batch<IC>(&mut self, tile_cs: IC)
    where
        IC: IntoIterator<Item = ([isize; N], [isize; N])> + Send + 'static,
    {
        self.add(SwapTileBatch::<L, IC, N> {
            tile_cs,
            label: std::marker::PhantomData,
        });
    }

    /// Manually spawn a chunk entity, note that this will overwrite and despawn existing chunks at this location.
    pub fn spawn_chunk<T>(&mut self, chunk_c: [isize; N], bundle: T) -> EntityCommands<'_>
    where
        T: Bundle + 'static,
    {
        let chunk_id = self.spawn(bundle).id();
        self.add(SpawnChunk::<L, N> {
            chunk_c,
            chunk_id,
            label: std::marker::PhantomData,
        });
        self.entity(chunk_id)
    }

    /// Spawns chunks from the given iterator using the given function.
    /// This will despawn any chunks (and their tiles) that already exists in this coordinate
    pub fn spawn_chunk_batch_with<F, B, IC>(&mut self, chunk_cs: IC, bundle_f: F)
    where
        F: Fn([isize; N]) -> B + Send + 'static,
        B: Bundle + Send + 'static,
        IC: IntoIterator<Item = [isize; N]> + Send + 'static,
    {
        self.add(SpawnChunkBatch::<L, F, B, IC, N> {
            chunk_cs,
            bundle_f,
            label: std::marker::PhantomData,
        });
    }

    /// Recursively despawn a chunk and all it's tiles.
    pub fn despawn_chunk(&mut self, chunk_c: [isize; N]) -> &mut Self {
        self.add(DespawnChunk::<L, N> {
            chunk_c,
            label: std::marker::PhantomData,
        });
        self
    }

    /// Despawns chunks (and their tiles) from the given iterator.
    pub fn despawn_chunk_batch<IC>(&mut self, chunk_cs: IC)
    where
        IC: IntoIterator<Item = [isize; N]> + Send + 'static,
    {
        self.add(DespawnChunkBatch::<L, IC, N> {
            chunk_cs,
            label: std::marker::PhantomData,
        });
    }

    /// Spawn a new map, overwriting any old maps found.
    pub fn spawn_map<T>(&mut self, bundle: T) -> EntityCommands<'_>
    where
        T: Bundle + 'static,
    {
        let map_id = self.spawn(bundle).id();
        self.add(SpawnMap::<L, N> {
            map_id,
            label: PhantomData,
        });
        self.entity(map_id)
    }

    /// Recursively despawns a map and all it's chunks and tiles.
    pub fn despawn_map(&mut self) -> &mut Self {
        self.add(DespawnMap::<L, N> { label: PhantomData });
        self
    }
}

/// Spawns a chunk in the world if needed, inserts the info into the map, and returns
/// and id for reinsertion
#[inline]
fn spawn_or_remove_chunk<L, const N: usize>(
    world: &mut World,
    map: &mut TileMap<N>,
    map_id: Entity,
    chunk_c: [isize; N],
) -> (Entity, Chunk)
where
    L: TileMapLabel + Send + 'static,
{
    if let Some(chunk_info) = remove_chunk::<N>(world, map, chunk_c) {
        chunk_info
    } else {
        let chunk_id = world
            .spawn((
                ChunkCoord::from(chunk_c),
                MapLabel::<L>::default(),
                InMap(map_id),
            ))
            .id();
        map.chunks.insert(chunk_c.into(), chunk_id);
        (chunk_id, Chunk::new(L::CHUNK_SIZE.pow(N as u32)))
    }
}

/// Removes a chunk from the world if it exists, and returns the info to reinsert it.
#[inline]
fn remove_chunk<const N: usize>(
    world: &mut World,
    map: &mut TileMap<N>,
    chunk_c: [isize; N],
) -> Option<(Entity, Chunk)> {
    map.chunks
        .get::<ChunkCoord<N>>(&chunk_c.into())
        .cloned()
        .and_then(|chunk_id| world.get_entity_mut(chunk_id))
        .map(|mut chunk_e| (chunk_e.id(), chunk_e.take::<Chunk>().unwrap()))
}

/// Takes the map out of the world or spawns a new one and returns the entity id to return the map to.
#[inline]
fn spawn_or_remove_map<L, const N: usize>(world: &mut World) -> (Entity, TileMap<N>)
where
    L: TileMapLabel + Send + 'static,
{
    let map_info = remove_map::<L, N>(world);
    if let Some(map_info) = map_info {
        map_info
    } else {
        (
            world.spawn(MapLabel::<L>::default()).id(),
            TileMap::<N>::with_chunk_size(L::CHUNK_SIZE),
        )
    }
}

/// Takes the map out of the world if it exists.
#[inline]
fn remove_map<L, const N: usize>(world: &mut World) -> Option<(Entity, TileMap<N>)>
where
    L: TileMapLabel + Send + 'static,
{
    world
        .query_filtered::<Entity, (With<TileMap<N>>, With<MapLabel<L>>)>()
        .get_single_mut(world)
        .ok()
        .map(|map_id| {
            (
                map_id,
                world
                    .get_entity_mut(map_id)
                    .unwrap()
                    .take::<TileMap<N>>()
                    .unwrap(),
            )
        })
}

/// Inserts a tile into the world
pub fn insert_tile<L, const N: usize>(world: &mut World, tile_c: [isize; N], tile_id: Entity)
where
    L: TileMapLabel + Send + 'static,
{
    // Take the map out and get the id to reinsert it
    let (map_id, mut map) = spawn_or_remove_map::<L, N>(world);

    // Take the chunk out and get the id to reinsert it
    let chunk_c = calculate_chunk_coordinate(tile_c, L::CHUNK_SIZE);
    let (chunk_id, mut chunk) = spawn_or_remove_chunk::<L, N>(world, &mut map, map_id, chunk_c);

    // Insert the tile
    let tile_i = calculate_tile_index(tile_c, L::CHUNK_SIZE);

    if let Some(tile) = chunk.tiles.get_mut(tile_i) {
        if let Some(old_tile_id) = tile.replace(tile_id) {
            world.despawn(old_tile_id);
        }
    }

    world.get_entity_mut(tile_id).unwrap().insert((
        TileIndex::from(tile_i),
        TileCoord::<N>::new(tile_c),
        MapLabel::<L>::default(),
        InChunk(chunk_id),
    ));

    world.get_entity_mut(chunk_id).unwrap().insert(chunk);
    world.get_entity_mut(map_id).unwrap().insert(map);
}

/// Take a tile from the world.
pub fn take_tile<L, const N: usize>(world: &mut World, tile_c: [isize; N]) -> Option<Entity>
where
    L: TileMapLabel + Send + 'static,
{
    // Get the map or return
    let (map_id, mut map) = remove_map::<L, N>(world)?;

    // Get the old chunk or return
    let chunk_c = calculate_chunk_coordinate(tile_c, L::CHUNK_SIZE);
    let (chunk_id, mut chunk) =
        if let Some(chunk_info) = remove_chunk::<N>(world, &mut map, chunk_c) {
            chunk_info
        } else {
            world.get_entity_mut(map_id).unwrap().insert(map);
            return None;
        };

    // Remove the old entity or return if the old entity is already deleted
    let tile_i = calculate_tile_index(tile_c, L::CHUNK_SIZE);

    let tile = if let Some(mut tile_e) = chunk
        .tiles
        .get_mut(tile_i)
        .and_then(|tile| tile.take())
        .and_then(|tile_id| world.get_entity_mut(tile_id))
    {
        tile_e.remove::<(TileIndex, TileCoord, MapLabel<L>, InChunk)>();
        let tile_id = tile_e.id();
        Some(tile_id)
    } else {
        None
    };

    world.get_entity_mut(chunk_id).unwrap().insert(chunk);
    world.get_entity_mut(map_id).unwrap().insert(map);
    tile
}

/// Inserts a list of entities into the corresponding tiles of a given tile map
pub fn insert_tile_batch<L, const N: usize>(
    world: &mut World,
    tiles: impl IntoIterator<Item = ([isize; N], Entity)>,
) where
    L: TileMapLabel + Send + 'static,
{
    let chunked_tiles = tiles
        .into_iter()
        .group_by(|(tile_c, _)| calculate_chunk_coordinate(*tile_c, L::CHUNK_SIZE));

    // Remove the map, or spawn an entity to hold the map, then create an empty map
    let (map_id, mut map) = spawn_or_remove_map::<L, N>(world);

    // Get the chunks and entities from the map
    let tiles_with_chunk = Vec::from_iter(chunked_tiles.into_iter().map(|(chunk_c, tiles)| {
        let (chunk_id, chunk) = spawn_or_remove_chunk::<L, N>(world, &mut map, map_id, chunk_c);
        (chunk_id, chunk, tiles)
    }));

    for (chunk_id, mut chunk, tiles) in tiles_with_chunk {
        for (tile_c, tile_id) in tiles {
            let tile_i = calculate_tile_index(tile_c, L::CHUNK_SIZE);

            if let Some(tile) = chunk.tiles.get_mut(tile_i) {
                if let Some(old_tile_id) = tile.replace(tile_id) {
                    world.despawn(old_tile_id);
                }
            }

            world.get_entity_mut(tile_id).unwrap().insert((
                TileIndex::from(tile_i),
                TileCoord::<N>::new(tile_c),
                MapLabel::<L>::default(),
                InChunk(chunk_id),
            ));
        }

        world.get_entity_mut(chunk_id).unwrap().insert(chunk);
    }

    world.get_entity_mut(map_id).unwrap().insert(map);
}

/// Removes the tiles from the tile map, returning the tile coordinates removed and their corresponding entities.
pub fn take_tile_batch<L, const N: usize>(
    world: &mut World,
    tiles: impl IntoIterator<Item = [isize; N]>,
) -> Vec<([isize; N], Entity)>
where
    L: TileMapLabel + Send + 'static,
{
    // Group tiles by chunk
    let chunked_tiles = tiles
        .into_iter()
        .group_by(|tile_c| calculate_chunk_coordinate(*tile_c, L::CHUNK_SIZE));

    // Remove the map, or return if it doesn't exist
    let (map_id, mut map) = if let Some(map_info) = remove_map::<L, N>(world) {
        map_info
    } else {
        return Vec::new();
    };

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
            let tile_i = calculate_tile_index(tile_c, L::CHUNK_SIZE);

            if let Some(mut tile_e) = chunk
                .tiles
                .get_mut(tile_i)
                .and_then(|tile| tile.take())
                .and_then(|tile_id| world.get_entity_mut(tile_id))
            {
                tile_e.remove::<(TileIndex, TileCoord, MapLabel<L>, InChunk)>();
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
pub fn insert_chunk<L, const N: usize>(world: &mut World, chunk_c: [isize; N], chunk_id: Entity)
where
    L: TileMapLabel + Send + 'static,
{
    let (map_id, mut map) = spawn_or_remove_map::<L, N>(world);

    // Despawn the chunk if it exists
    if let Some(old_chunk) = take_chunk_despawn_tiles_inner::<L, N>(world, chunk_c, &mut map) {
        world.despawn(old_chunk);
    }

    world.get_entity_mut(chunk_id).unwrap().insert((
        Chunk::new(L::CHUNK_SIZE.pow(N as u32)),
        ChunkCoord::from(chunk_c),
        MapLabel::<L>::default(),
        InMap(map_id),
    ));
    map.chunks.insert(chunk_c.into(), chunk_id);

    world.entity_mut(map_id).insert(map);
}

/// Remove the chunk from the map without despawning it.
/// # Note
/// This does not despawn or remove the tile entities, and reinsertion of this entity will not recreate the link to the chunk's tiles.
/// If you wish to take the chunk and delete it's underlying tiles, use (take_chunk_despawn_tiles)[`take_chunk_despawn_tiles`]
pub fn take_chunk<L, const N: usize>(world: &mut World, chunk_c: [isize; N]) -> Option<Entity>
where
    L: TileMapLabel + Send + 'static,
{
    // Get the map or return
    let (map_id, mut map) = remove_map::<L, N>(world)?;

    // Get the old chunk or return
    let chunk_id = if let Some(mut chunk_e) = map
        .chunks
        .remove::<ChunkCoord<N>>(&chunk_c.into())
        .and_then(|chunk_id| world.get_entity_mut(chunk_id))
    {
        let (chunk, _, _, _) = chunk_e
            .take::<(Chunk, ChunkCoord, MapLabel<L>, InMap)>()
            .unwrap();
        let chunk_id = chunk_e.id();
        for tile_id in chunk.tiles.into_iter().flatten() {
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
pub fn take_chunk_despawn_tiles<L, const N: usize>(
    world: &mut World,
    chunk_c: [isize; N],
) -> Option<Entity>
where
    L: TileMapLabel + Send + 'static,
{
    // Get the map or return
    let (map_id, mut map) = remove_map::<L, N>(world)?;

    let chunk_id = take_chunk_despawn_tiles_inner::<L, N>(world, chunk_c, &mut map);

    world.entity_mut(map_id).insert(map);

    chunk_id
}

pub(crate) fn take_chunk_despawn_tiles_inner<L, const N: usize>(
    world: &mut World,
    chunk_c: [isize; N],
    mut map: &mut TileMap<N>,
) -> Option<Entity>
where
    L: TileMapLabel + Send + 'static,
{
    // Get the old chunk or return
    let chunk_id = if let Some(mut chunk_e) = map
        .chunks
        .remove::<ChunkCoord<N>>(&chunk_c.into())
        .and_then(|chunk_id| world.get_entity_mut(chunk_id))
    {
        let (chunk, _, _, _) = chunk_e
            .take::<(Chunk, ChunkCoord, MapLabel<L>, InMap)>()
            .unwrap();
        let chunk_id = chunk_e.id();
        for tile_id in chunk.tiles.into_iter().flatten() {
            world.despawn(tile_id);
        }
        Some(chunk_id)
    } else {
        None
    };
    chunk_id
}

/// Inserts a list of entities into map and treats them as chunks
pub fn insert_chunk_batch<L, const N: usize>(
    world: &mut World,
    chunks: impl IntoIterator<Item = ([isize; N], Entity)>,
) where
    L: TileMapLabel + Send + 'static,
{
    // Remove the map, or spawn an entity to hold the map, then create an empty map
    let (map_id, mut map) = spawn_or_remove_map::<L, N>(world);

    // Get the chunks and entities from the map
    for (chunk_c, chunk_id) in chunks.into_iter() {
        if let Some(old_chunk) = take_chunk_despawn_tiles_inner::<L, N>(world, chunk_c, &mut map) {
            world.despawn(old_chunk);
        }

        world.get_entity_mut(chunk_id).unwrap().insert((
            Chunk::new(L::CHUNK_SIZE.pow(N as u32)),
            ChunkCoord::from(chunk_c),
            MapLabel::<L>::default(),
            InMap(map_id),
        ));
        map.chunks.insert(chunk_c.into(), chunk_id);
    }

    world.get_entity_mut(map_id).unwrap().insert(map);
}

/// Removes the chunks from the tile map, returning the chunk coordinates removed and their corresponding entities.
/// # Note
/// This does not despawn or remove the tile entities, and reinsertion of this entity will not recreate the link to the chunk's tiles.
/// If you wish to take the chunk and delete it's underlying tiles, use (take_chunk_batch_despawn_tiles)[`take_chunk_batch_despawn_tiles`]
pub fn take_chunk_batch<L, const N: usize>(
    world: &mut World,
    chunks: impl IntoIterator<Item = [isize; N]>,
) -> Vec<([isize; N], Entity)>
where
    L: TileMapLabel + Send + 'static,
{
    // Remove the map, or return if it doesn't exist
    let (map_id, mut map) = if let Some(map_info) = remove_map::<L, N>(world) {
        map_info
    } else {
        return Vec::new();
    };

    let mut chunk_ids = Vec::new();

    for chunk_c in chunks.into_iter() {
        // Get the old chunk or return
        if let Some(mut chunk_e) = map
            .chunks
            .remove::<ChunkCoord<N>>(&chunk_c.into())
            .and_then(|chunk_id| world.get_entity_mut(chunk_id))
        {
            let (chunk, _, _, _) = chunk_e
                .take::<(Chunk, ChunkCoord, MapLabel<L>, InMap)>()
                .unwrap();
            let chunk_id = chunk_e.id();
            for tile_id in chunk.tiles.into_iter().flatten() {
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
pub fn take_chunk_batch_despawn_tiles<L, const N: usize>(
    world: &mut World,
    chunks: impl IntoIterator<Item = [isize; N]>,
) -> Vec<([isize; N], Entity)>
where
    L: TileMapLabel + Send + 'static,
{
    // Remove the map, or return if it doesn't exist
    let (map_id, mut map) = if let Some(map_info) = remove_map::<L, N>(world) {
        map_info
    } else {
        return Vec::new();
    };

    let mut chunk_ids = Vec::new();

    for chunk_c in chunks.into_iter() {
        // Get the old chunk or return
        if let Some(mut chunk_e) = map
            .chunks
            .remove::<ChunkCoord<N>>(&chunk_c.into())
            .and_then(|chunk_id| world.get_entity_mut(chunk_id))
        {
            let (chunk, _, _, _) = chunk_e
                .take::<(Chunk, ChunkCoord, MapLabel<L>, InMap)>()
                .unwrap();
            let chunk_id = chunk_e.id();
            for tile_id in chunk.tiles.into_iter().flatten() {
                world.despawn(tile_id);
            }
            chunk_ids.push((chunk_c, chunk_id));
        };
    }

    world.get_entity_mut(map_id).unwrap().insert(map);
    chunk_ids
}

/// Insert the given entity and have it be treated as the given map.
/// # Note
/// This will despawn any existing map with this label.
pub fn insert_map<L, const N: usize>(world: &mut World, map_id: Entity)
where
    L: TileMapLabel + Send + 'static,
{
    let map_info = remove_map::<L, N>(world);
    DespawnMap::<L, N>::default().apply(world);
    world.entity_mut(map_id).insert((
        MapLabel::<L>::default(),
        TileMap::<N>::with_chunk_size(L::CHUNK_SIZE),
    ));
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
