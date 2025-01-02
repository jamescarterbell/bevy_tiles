use std::{cmp::Eq, hash::Hash};

use bevy::{
    ecs::system::EntityCommands,
    prelude::{Bundle, Commands, Entity, World},
    utils::{hashbrown::hash_map::Entry, HashMap},
};

mod tile_batch;
mod tile_single;

use bevy_tiles::{commands::TileMapCommands, queries::TileComponent};
use tile_batch::*;
use tile_single::*;

use crate::EntityTile;

/// ECS extensions for bevy_tiles.
pub trait TileMapCommandsECSExt<const N: usize> {
    /// Spawns a tile and returns a handle to the underlying entity.
    /// This will despawn any tile that already exists in this coordinate
    fn spawn_tile(&mut self, tile_c: impl Into<[i32; N]>, bundle: impl Bundle) -> EntityCommands;

    /// Spawns a tile and returns a handle to the underlying entity.
    /// This will despawn any tile that already exists in this coordinate
    fn spawn_tile_batch(
        &mut self,
        tile_cs: impl IntoIterator<Item = [i32; N]> + Send + 'static,
        bundles: impl Bundle + Clone,
    ) -> &mut Self;

    /// Despawns a tile .
    fn despawn_tile(&mut self, tile_c: impl Into<[i32; N]>) -> &mut Self;

    /// Moves a tile entities.
    fn move_tile(&mut self, old_c: impl Into<[i32; N]>, new_c: impl Into<[i32; N]>) -> &mut Self;

    /// Swaps two tile entities.
    fn swap_tiles(
        &mut self,
        tile_c_1: impl Into<[i32; N]>,
        tile_c_2: impl Into<[i32; N]>,
    ) -> &mut Self;
}

impl<'a, const N: usize> TileMapCommandsECSExt<N> for TileMapCommands<'a, N> {
    /// Spawns a tile and returns a handle to the underlying entity.
    /// This will despawn any tile that already exists at the coordinate.
    fn spawn_tile(&mut self, tile_c: impl Into<[i32; N]>, bundle: impl Bundle) -> EntityCommands {
        let tile_c = tile_c.into();
        let tile_id = self.commands().spawn(bundle).id();
        let map_id = self.id();
        self.commands().queue(SpawnTile::<N> {
            map_id,
            tile_c,
            tile_id: EntityTile(tile_id),
        });
        self.commands_mut().entity(tile_id)
    }

    /// Despawns a tile.
    fn despawn_tile(&mut self, tile_c: impl Into<[i32; N]>) -> &mut Self {
        let tile_c = tile_c.into();
        let map_id = self.id();
        self.commands().queue(DespawnTile { map_id, tile_c });

        self
    }

    /// Moves a tile from one coordinate to another, overwriting and despawning any tile in the new coordinate.
    fn move_tile(&mut self, old_c: impl Into<[i32; N]>, new_c: impl Into<[i32; N]>) -> &mut Self {
        let old_c = old_c.into();
        let new_c = new_c.into();
        let map_id = self.id();
        self.commands().queue(MoveTile {
            map_id,
            old_c,
            new_c,
        });

        self
    }

    /// Swaps two tiles if both exist, or moves one tile if the other doesn't exist.
    fn swap_tiles(
        &mut self,
        tile_c_0: impl Into<[i32; N]>,
        tile_c_1: impl Into<[i32; N]>,
    ) -> &mut Self {
        let tile_c_0 = tile_c_0.into();
        let tile_c_1 = tile_c_1.into();
        let map_id = self.id();
        self.commands().queue(SwapTile {
            map_id,
            tile_c_0,
            tile_c_1,
        });

        self
    }

    fn spawn_tile_batch(
        &mut self,
        tile_cs: impl IntoIterator<Item = [i32; N]> + Send + 'static,
        tile_b: impl Bundle + Clone,
    ) -> &mut Self {
        let map_id = self.id();
        let commands = self.commands_mut();
        commands.queue(SpawnTileBatch::<_, _, N> {
            map_id,
            tile_cs,
            tile_b,
        });
        self
    }
}
