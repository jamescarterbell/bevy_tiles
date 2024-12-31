use std::{cmp::Eq, hash::Hash};

use bevy::{
    ecs::system::EntityCommands,
    prelude::{Bundle, Commands, Entity, World},
    utils::{hashbrown::hash_map::Entry, HashMap},
};

// mod tile_batch;
mod tile_single;

use bevy_tiles::{commands::TileMapCommands, queries::TileBundle};
// use tile_batch::*;
use tile_single::*;

/// ECS extensions for bevy_tiles.
pub trait TileMapCommandsECSExt<const N: usize> {
    /// Spawns a tile and returns a handle to the underlying entity.
    /// This will despawn any tile that already exists in this coordinate
    fn spawn_tile(&mut self, tile_c: impl Into<[i32; N]>, bundle: impl Bundle) -> EntityCommands;

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
        self.commands().queue(SpawnTile {
            map_id,
            tile_c,
            tile_id,
        });
        self.commands_mut().entity(tile_id)
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
    fn despawn_tile(&mut self, tile_c: impl Into<[i32; N]>) -> &mut Self {
        let tile_c = tile_c.into();
        let map_id = self.id();
        self.commands().queue(DespawnTile { map_id, tile_c });

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

    // /// Move tiles from the first coordinate to the second coordinate, despawning
    // /// any tile found in the second coordinate.
    // pub fn move_tile_batch<IC>(&mut self, tile_cs: IC) -> &mut Self
    // where
    //     IC: IntoIterator<Item = ([i32; N], [i32; N])> + Send + 'static,
    // {
    //     self.commands.move_tile_batch(self.map_id, tile_cs);
    //     self
    // }

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

    // /// Swap tiles from the first coordinate and the second coordinate
    // pub fn swap_tile_batch<IC>(&mut self, tile_cs: IC) -> &mut Self
    // where
    //     IC: IntoIterator<Item = ([i32; N], [i32; N])> + Send + 'static,
    // {
    //     self.commands.swap_tile_batch(self.map_id, tile_cs);
    //     self
    // }
}
