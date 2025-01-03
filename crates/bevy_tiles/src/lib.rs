//! A general purpose grided entity library meant to support tilemap libraries,
//! or other libraries that require accessing entities in a grid based manner.
//!
//! The goal is to keep the API surface as simple and intuitive as possible,
//! and to avoid deferred operations/states where possible to make the structures more intuitive work with.
//!  (ex: an update in one system should be seen by the following system, not the following frame.)
//!
//! Tilemaps consist of two main component types:
//! * Entities with a [`crate::maps::TileMap`] component are the root entity of a TileMap.
//!     - Adding a [`crate::maps::UseTransforms`] component will add transforms to the TileMap and any
//!       spawned children of the map (chunks and tiles).
//!     - Adding a [`crate::maps::TileDims`] component will configure the size of the tile for chunk spacing.
//!     - Adding a [`crate::maps::TileSpacing`] component will configure the spacing between tiles for chunk spacing.
//! * When adding tiles to a tilemap, if one does not exist for that tiles chunk, an entity with a [`crate::chunks::ChunkData<T>`] for the given
//!   tile data will be spawned.  The chunk data component is a flat vector containing all the tile data for a given chunk.
//!     - If the parent map has [`crate::maps::UseTransforms`] then the chunk will be spawned with a transform configured using the
//!       [`crate::maps::TileDims`] and [`crate::maps::TileSpacing`] components.
//!
//! In this crate, there are no tile entities! Chunks act as psuedo component stores, and tiles as psuedo components.

#![deny(missing_docs)]

use bevy::app::Plugin;

/// Provides chunk level utilities.
pub mod chunks;
/// Provides commands for interacting with tilemaps.
pub mod commands;
/// Provides helper functions for interacting with coordiantes.
pub mod coords;
/// Provides map level utilities.
pub mod maps;
/// Provides traits for accessing tile data.
pub mod queries;
/// Provides tile level utilities.
pub mod tiles;

/// Helper aliases for working with 2d grids
pub mod tiles_2d {
    use bevy::ecs::system::Commands;

    /// 2d [crate::tiles::TileMapQuery] alias.
    pub type TileMapQuery<'w, 's, Q> = crate::tiles::TileMapQuery<'w, 's, Q, 2>;

    /// 2d [crate::chunks::ChunkCoord] alias.
    pub type ChunkCoord = crate::chunks::ChunkCoord<2>;

    /// 2d [crate::chunks::ChunkMapQuery] alias.
    pub type ChunkMapQuery<'w, 's, Q, F = ()> = crate::chunks::ChunkMapQuery<'w, 's, Q, F, 2>;

    /// 2d [crate::commands::TileMapCommands] alias.
    pub type TileMapCommands<'a, const N: usize> = crate::commands::TileMapCommands<'a, 2>;

    /// 2d [crate::commands::TileCommandExt] alias.
    pub trait TileCommandExt<'w, 's>: crate::commands::TileCommandExt<'w, 's, 2> {}

    impl<'w, 's> TileCommandExt<'w, 's> for Commands<'w, 's> {}

    /// 2d [crate::maps::TileDims] alias.
    pub type TileDims = crate::maps::TileDims<2>;

    /// 2d [crate::maps::TileSpacing] alias.
    pub type TileSpacing = crate::maps::TileSpacing<2>;
}

/// Helper aliases for working with 2d grids
pub mod tiles_3d {
    use bevy::ecs::system::Commands;

    /// 3d [crate::tiles::TileMapQuery] alias.
    pub type TileMapQuery<'w, 's, Q> = crate::tiles::TileMapQuery<'w, 's, Q, 3>;

    /// 3d [crate::chunks::ChunkCoord] alias.
    pub type ChunkCoord = crate::chunks::ChunkCoord<3>;

    /// 3d [crate::chunks::ChunkMapQuery] alias.
    pub type ChunkMapQuery<'w, 's, Q, F = ()> = crate::chunks::ChunkMapQuery<'w, 's, Q, F, 3>;

    /// 3d [crate::commands::TileMapCommands] alias.
    pub type TileMapCommands<'a, const N: usize> = crate::commands::TileMapCommands<'a, 3>;

    /// 3d [crate::commands::TileCommandExt] alias.
    pub trait TileCommandExt<'w, 's>: crate::commands::TileCommandExt<'w, 's, 3> {}

    impl<'w, 's> TileCommandExt<'w, 's> for Commands<'w, 's> {}

    /// 3d [crate::maps::TileDims] alias.
    pub type TileDims = crate::maps::TileDims<3>;

    /// 3d [crate::maps::TileSpacing] alias.
    pub type TileSpacing = crate::maps::TileSpacing<3>;
}

/// Adds Tiles dependencies to the App.
pub struct TilesPlugin;

impl Plugin for TilesPlugin {
    fn build(&self, _app: &mut bevy::prelude::App) {}
}
