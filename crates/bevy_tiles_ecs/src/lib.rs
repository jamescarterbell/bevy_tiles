//! A general purpose grided entity library meant to support tilemap libraries,
//! or other libraries that require accessing entities in a grid based manner.
//!
//! The goal is to keep the API surface as simple and intuitive as possible,
//! and to avoid deferred operations/states where possible to make the structures more intuitive work with.
//!  (ex: an update in one system should be seen by the following system, not the following frame.)

#![deny(missing_docs)]

use bevy::app::Plugin;

/// Provides commands for interacting with tilemaps.
pub mod commands;
/// The entity tracking tile component.
pub mod entity_tile;
/// Provides tile level utilities.
pub mod tiles;

pub(crate) use entity_tile::EntityTile;

/// Helper aliases for working with 2d grids
pub mod tiles_2d {
    use bevy_tiles::commands::TileMapCommands;

    /// 2d [crate::tiles::TileCoord] alias.
    pub type TileCoord = crate::entity_tile::TileCoord<2>;

    /// 2d [crate::tiles::TileEntityMapQuery] alias.
    pub type TileEntityMapQuery<'w, 's, Q, F> = crate::tiles::TileEntityMapQuery<'w, 's, Q, F, 2>;

    /// 2d [crate::commands::TileCommandExt] alias.
    pub trait TileMapCommandsECSExt: crate::commands::TileMapCommandsECSExt<2> {}

    impl<'a> TileMapCommandsECSExt for TileMapCommands<'a, 2> {}
}

/// Helper aliases for working with 3d grids
pub mod tiles_3d {
    use bevy_tiles::commands::TileMapCommands;

    /// 2d [crate::tiles::TileCoord] alias.
    pub type TileCoord = crate::entity_tile::TileCoord<3>;

    /// 2d [crate::tiles::TileEntityMapQuery] alias.
    pub type TileEntityMapQuery<'w, 's, Q, F> = crate::tiles::TileEntityMapQuery<'w, 's, Q, F, 3>;

    /// 2d [crate::commands::TileCommandExt] alias.
    pub trait TileMapCommandsECSExt: crate::commands::TileMapCommandsECSExt<3> {}

    impl<'a> TileMapCommandsECSExt for TileMapCommands<'a, 3> {}
}

/// Adds Tiles dependencies to the App.
pub struct TilesPlugin;

impl Plugin for TilesPlugin {
    fn build(&self, _app: &mut bevy::prelude::App) {}
}
