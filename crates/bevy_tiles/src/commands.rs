use std::ops::{Deref, DerefMut};

use crate::{
    chunks::{ChunkCoord, ChunkTypes, InMap},
    coords::{calculate_chunk_coordinate, calculate_tile_index},
    maps::{TileDims, TileMap, TileSpacing, UseTransforms},
    queries::TileComponent,
};

use bevy::{
    ecs::system::EntityCommands,
    math::Vec3,
    prelude::{
        BuildChildren, Bundle, Commands, Deref, DerefMut, DespawnRecursiveExt, Entity,
        EntityWorldMut, InheritedVisibility, Transform, Visibility, World,
    },
    utils::hashbrown::{hash_map::Entry, HashMap},
};

// mod chunk_batch;
mod chunk_single;
// mod tile_batch;
mod tile_single;

// use chunk_batch::*;
use chunk_single::*;
// use tile_batch::*;
use tile_single::*;

/// Applies commands to a specific tile map.
#[derive(Deref, DerefMut)]
pub struct TileMapCommands<'a, const N: usize> {
    commands: EntityCommands<'a>,
}

impl<'a, const N: usize> TileMapCommands<'a, N> {
    /// Spawns a tile and returns a handle to the underlying entity.
    /// This will despawn any tile that already exists in this coordinate
    pub fn insert_tile<B: TileComponent>(&mut self, tile_c: impl Into<[i32; N]>, bundle: B) {
        let tile_c = tile_c.into();
        let id = self.commands.id();
        self.commands.commands().spawn_tile(id, tile_c, bundle);
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
    pub fn remove_tile<B: TileComponent>(&mut self, tile_c: impl Into<[i32; N]>) -> &mut Self {
        let tile_c = tile_c.into();
        let id = self.commands.id();
        self.commands.commands().remove_tile::<B>(id, tile_c);
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

    /// Manually spawn a chunk entity, note that this will overwrite and despawn existing chunks at this location.
    pub fn spawn_chunk(&mut self, chunk_c: impl Into<[i32; N]>) {
        let chunk_c = chunk_c.into();
        let id = self.commands.id();
        self.commands.commands().spawn_chunk(id, chunk_c)
    }

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

    /// Recursively despawn a chunk and all it's tiles.
    pub fn despawn_chunk(&mut self, chunk_c: impl Into<[i32; N]>) -> &mut Self {
        let chunk_c = chunk_c.into();
        let map_id = self.id();
        self.commands().despawn_chunk(map_id, chunk_c);
        self
    }

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
}

/// Helper method for creating map specific commands.
pub trait TileCommandExt<'w, 's, const N: usize> {
    /// Gets [TileMapCommands] to apply commands at the tile map level.
    fn tile_map(&mut self, map_id: Entity) -> Option<TileMapCommands<'_, N>>;

    /// Spawns a tile and returns a handle to the underlying entity.
    /// This will despawn any tile that already exists in this coordinate
    fn spawn_tile<B: TileComponent>(&mut self, map_id: Entity, tile_c: [i32; N], bundle: B);

    // /// Spawns tiles from the given iterator using the given function.
    // /// This will despawn any tile that already exists in this coordinate
    // fn spawn_tile_batch<F, B, IC>(&mut self, map_id: Entity, tile_cs: IC, bundle_f: F)
    // where
    //     F: Fn([i32; N]) -> B + Send + 'static,
    //     B: Bundle + Send + 'static,
    //     IC: IntoIterator<Item = [i32; N]> + Send + 'static;

    /// Despawns a tile.
    fn remove_tile<B: TileComponent>(&mut self, map_id: Entity, tile_c: [i32; N]) -> &mut Self;

    // /// Despawns tiles from the given iterator.
    // fn despawn_tile_batch<IC>(&mut self, map_id: Entity, tile_cs: IC)
    // where
    //     IC: IntoIterator<Item = [i32; N]> + Send + 'static;

    /// Manually spawn a chunk entity, note that this will overwrite and despawn existing chunks at this location.
    fn spawn_chunk(&mut self, map_id: Entity, chunk_c: [i32; N]);

    // /// Spawns chunks from the given iterator using the given function.
    // /// This will despawn any chunks (and their tiles) that already exists in this coordinate
    // fn spawn_chunk_batch_with<F, B, IC>(&mut self, map_id: Entity, chunk_cs: IC, bundle_f: F)
    // where
    //     F: Fn([i32; N]) -> B + Send + 'static,
    //     B: Bundle + Send + 'static,
    //     IC: IntoIterator<Item = [i32; N]> + Send + 'static;

    /// Recursively despawn a chunk and all it's tiles.
    fn despawn_chunk(&mut self, map_id: Entity, chunk_c: [i32; N]) -> &mut Self;

    // /// Despawns chunks (and their tiles) from the given iterator.
    // fn despawn_chunk_batch<IC>(&mut self, map_id: Entity, chunk_cs: IC)
    // where
    //     IC: IntoIterator<Item = [i32; N]> + Send + 'static;

    /// Spawn a new map.
    fn spawn_map(&mut self, chunk_size: usize) -> TileMapCommands<'_, N>;

    /// Recursively despawns a map and all it's chunks and tiles.
    fn despawn_map(&mut self, map_id: Entity) -> &mut Self;
}

impl<'w, 's, const N: usize> TileCommandExt<'w, 's, N> for Commands<'w, 's> {
    fn tile_map(&mut self, map_id: Entity) -> Option<TileMapCommands<'_, N>> {
        self.get_entity(map_id)
            .map(|commands| TileMapCommands { commands })
    }

    /// Spawns a tile and returns a handle to the underlying entity.
    /// This will despawn any tile that already exists in this coordinate
    fn spawn_tile<B: TileComponent>(&mut self, map_id: Entity, tile_c: [i32; N], bundle: B) {
        self.queue(InsertTile::<B, N> {
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
    fn remove_tile<B: TileComponent>(&mut self, map_id: Entity, tile_c: [i32; N]) -> &mut Self {
        self.queue(RemoveTile::<B, N> {
            map_id,
            tile_c,
            bundle: Default::default(),
        });
        self
    }

    /// Manually spawn a chunk entity, note that this will overwrite and despawn existing chunks at this location.
    fn spawn_chunk(&mut self, map_id: Entity, chunk_c: [i32; N]) {
        self.queue(SpawnChunk::<N> { map_id, chunk_c });
    }

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

    /// Recursively despawn a chunk and all it's tiles.
    fn despawn_chunk(&mut self, map_id: Entity, chunk_c: [i32; N]) -> &mut Self {
        self.queue(DespawnChunk::<N> { map_id, chunk_c });
        self
    }

    // /// Despawns chunks (and their tiles) from the given iterator.
    // fn despawn_chunk_batch<IC>(&mut self, map_id: Entity, chunk_cs: IC)
    // where
    //     IC: IntoIterator<Item = [i32; N]> + Send + 'static,
    // {
    //     self.add(DespawnChunkBatch::<IC, N> { map_id, chunk_cs });
    // }

    /// Spawn a new map.
    fn spawn_map(&mut self, chunk_size: usize) -> TileMapCommands<'_, N> {
        TileMapCommands {
            commands: self.spawn((
                TileMap::<N>::with_chunk_size(chunk_size),
                Visibility::default(),
                InheritedVisibility::default(),
                Transform::default(),
            )),
        }
    }

    /// Recursively despawns a map and all it's chunks and tiles.
    fn despawn_map(&mut self, map_id: Entity) -> &mut Self {
        self.reborrow().entity(map_id).despawn_recursive();
        self
    }
}

/// Spawns a chunk in the world if needed, inserts the info into the map, and returns
/// and id for reinsertion
#[inline]
fn get_chunk<'a, const N: usize>(
    map: &'a mut TempRemoved<'_, TileMap<N>>,
    chunk_c: [i32; N],
) -> Option<EntityWorldMut<'a>> {
    let chunk_id = *map
        .get_chunks()
        .get::<ChunkCoord<N>>(&ChunkCoord(chunk_c))?;

    if map.world.entities().contains(chunk_id) {
        map.world.get_entity_mut(chunk_id).ok()
    } else {
        None
    }
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

    let (use_transforms, tile_dims, tile_spacing) = map
        .world
        .query::<(
            Option<&UseTransforms>,
            Option<&TileDims<N>>,
            Option<&TileSpacing<N>>,
        )>()
        .get(map.world, map.source)
        .unwrap();

    let (use_transforms, tile_dims, tile_spacing) = (
        use_transforms.cloned(),
        tile_dims.cloned(),
        tile_spacing.cloned(),
    );

    if let Some(chunk_id) = chunk_id {
        // Todo: Change this when NLL is fixed :)
        if map.world.entities().contains(chunk_id) {
            return map.world.get_entity_mut(chunk_id).unwrap();
        }
    }

    spawn_chunk(
        map,
        chunk_c,
        use_transforms.is_some(),
        tile_dims,
        tile_spacing,
    )
}

#[inline]
fn spawn_chunk<'a, const N: usize>(
    map: &'a mut TempRemoved<'_, TileMap<N>>,
    chunk_c: [i32; N],
    use_transforms: bool,
    tile_dims: Option<TileDims<N>>,
    tile_spacing: Option<TileSpacing<N>>,
) -> EntityWorldMut<'a> {
    let chunk_c = ChunkCoord(chunk_c);

    let chunk_id = match (use_transforms, tile_dims) {
        (true, Some(size)) => {
            let translation = match N {
                1 => Vec3::new(
                    calc_chunk_trans_dim(0, map.get_chunk_size(), chunk_c, size, tile_spacing),
                    0.0,
                    0.0,
                ),
                2 => Vec3::new(
                    calc_chunk_trans_dim(0, map.get_chunk_size(), chunk_c, size, tile_spacing),
                    calc_chunk_trans_dim(1, map.get_chunk_size(), chunk_c, size, tile_spacing),
                    0.0,
                ),
                3 => Vec3::new(
                    calc_chunk_trans_dim(0, map.get_chunk_size(), chunk_c, size, tile_spacing),
                    calc_chunk_trans_dim(1, map.get_chunk_size(), chunk_c, size, tile_spacing),
                    calc_chunk_trans_dim(2, map.get_chunk_size(), chunk_c, size, tile_spacing),
                ),
                _ => {
                    panic!("Can't use transforms on tilemaps with more than 3 dimensions :)");
                }
            };
            map.world
                .spawn((
                    Transform {
                        translation,
                        ..Default::default()
                    },
                    Visibility::default(),
                    InheritedVisibility::default(),
                    ChunkCoord(chunk_c.0),
                    InMap(map.source),
                    ChunkTypes::default(),
                ))
                .set_parent(map.source)
                .id()
        }
        (_, _) => map
            .world
            .spawn((
                ChunkCoord(chunk_c.0),
                InMap(map.source),
                ChunkTypes::default(),
            ))
            .set_parent(map.source)
            .id(),
    };

    map.get_chunks_mut().insert(chunk_c, chunk_id);
    map.world.get_entity_mut(chunk_id).unwrap()
}

#[inline]
fn calc_chunk_trans_dim<const N: usize>(
    dim: usize,
    chunk_dims: usize,
    chunk_c: ChunkCoord<N>,
    dims: TileDims<N>,
    spacing: Option<TileSpacing<N>>,
) -> f32 {
    let coord = chunk_dims as f32 * chunk_c.0[dim] as f32;
    dims.0[dim] * coord + spacing.map(|spacing| spacing.0[dim] * coord).unwrap_or(0.0)
}

/// Inserts a tile into the given map.
#[inline]
pub fn insert_tile<B: TileComponent, const N: usize>(
    map: &mut TempRemoved<'_, TileMap<N>>,
    tile_c: [i32; N],
    tile_bundle: B,
) -> Option<B> {
    let chunk_size = map.get_chunk_size();

    let (use_transforms, tile_dims, tile_spacing) = map
        .world
        .query::<(
            Option<&UseTransforms>,
            Option<&TileDims<N>>,
            Option<&TileSpacing<N>>,
        )>()
        .get(map.world, map.source)
        .unwrap();

    let (use_transforms, tile_dims, tile_spacing) = (
        use_transforms.cloned(),
        tile_dims.cloned(),
        tile_spacing.cloned(),
    );

    // Take the chunk out and get the id to reinsert it
    let chunk_c = calculate_chunk_coordinate(tile_c, chunk_size);
    let chunk = get_or_spawn_chunk::<N>(map, chunk_c);

    // Insert the tile
    let tile_i = calculate_tile_index(tile_c, chunk_size);

    tile_bundle.insert_tile_into_chunk::<N>(
        chunk,
        chunk_c,
        chunk_size,
        use_transforms.is_some(),
        tile_dims,
        tile_spacing,
        tile_c,
        tile_i,
    )
}

/// Inserts a batch of tiles into the given map.
/// # NOTE:
/// The bundle and coord iterators must be the same size!
#[inline]
pub fn insert_tile_batch<B: TileComponent, const N: usize>(
    map: &mut TempRemoved<'_, TileMap<N>>,
    tile_cs: impl IntoIterator<Item = [i32; N]>,
    tile_bundles: impl IntoIterator<Item = B>,
) -> impl Iterator<Item = B> {
    let chunk_size = map.get_chunk_size();
    let mut tiles = tile_bundles.into_iter();

    let mut chunk_cs = HashMap::new();

    for tile_c in tile_cs {
        let chunk_c = calculate_chunk_coordinate(tile_c, chunk_size);
        let tiles = match chunk_cs.entry(chunk_c) {
            Entry::Occupied(occupied_entry) => occupied_entry.into_mut(),
            Entry::Vacant(vacant_entry) => vacant_entry.insert(Vec::new()),
        };
        tiles.push((tile_c, calculate_tile_index(tile_c, chunk_size)));
    }

    let mut replaced_vals = Vec::new();

    let (use_transforms, tile_dims, tile_spacing) = map
        .world
        .query::<(
            Option<&UseTransforms>,
            Option<&TileDims<N>>,
            Option<&TileSpacing<N>>,
        )>()
        .get(map.world, map.source)
        .unwrap();

    let (use_transforms, tile_dims, tile_spacing) = (
        use_transforms.cloned(),
        tile_dims.cloned(),
        tile_spacing.cloned(),
    );

    for (chunk_c, tile_is) in chunk_cs {
        let chunk = get_or_spawn_chunk::<N>(map, chunk_c);
        for replaced in B::insert_tile_batch_into_chunk::<N>(
            &mut tiles,
            chunk,
            chunk_c,
            chunk_size,
            use_transforms.is_some(),
            tile_dims,
            tile_spacing,
            tile_is.into_iter(),
        ) {
            replaced_vals.push(replaced);
        }
    }
    replaced_vals.into_iter()
}

/// Removes a tile from the given map if it exists.
#[inline]
pub fn take_tile<B: TileComponent, const N: usize>(
    map: &mut TempRemoved<'_, TileMap<N>>,
    tile_c: [i32; N],
) -> Option<B> {
    let chunk_size = map.get_chunk_size();

    let chunk_c = calculate_chunk_coordinate(tile_c, chunk_size);
    let chunk_c = ChunkCoord::<N>(chunk_c);
    let chunk_id = map.get_chunks().get(&chunk_c)?;
    let mut chunk_e = map.world.get_entity_mut(*chunk_id).ok()?;

    // Insert the tile
    let tile_i = calculate_tile_index(tile_c, chunk_size);

    B::take_tile_from_chunk(&mut chunk_e, tile_i)
}

/// Temporarily removed bundle from the world.
pub struct TempRemoved<'w, T: Bundle> {
    value: Option<T>,
    world: &'w mut World,
    source: Entity,
}

impl<'w, T: Bundle> TempRemoved<'w, T> {
    /// Get the world this value was removed from.
    pub fn get_world_mut(&mut self) -> &mut World {
        self.world
    }
}

impl<'w, T: Bundle> Drop for TempRemoved<'w, T> {
    #[inline]
    fn drop(&mut self) {
        EntityWorldMut::insert(
            &mut self.world.get_entity_mut(self.source).unwrap(),
            self.value.take().unwrap(),
        );
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
            .ok()
            .and_then(|mut ent| ent.take::<T>().map(|val| (ent.id(), val)))
            .map(|(id, val)| TempRemoved {
                value: Some(val),
                world: self,
                source: id,
            })
    }
}
