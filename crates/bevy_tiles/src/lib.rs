//! A general purpose grided entity library meant to support tilemap libraries,
//! or other libraries that require accessing entities in a grid based manner.
//!
//! The goal is to keep the API surface as simple and intuitive as possible,
//! and to avoid deferred operations/states where possible to make the structures more intuitive work with.
//!  (ex: an update in one system should be seen by the following system, not the following frame.)

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
/// Provides tile level utilities.
pub mod tiles;

/// Provides most of what you need to get started.
pub mod prelude {
    pub use crate::commands::{TileCommandExt, TileCommands};

    pub use crate::chunks::*;
    pub use crate::coords::*;
    pub use crate::maps::*;
    pub use crate::tiles::*;
    pub use crate::TilesPlugin;
}

/// Adds Tiles dependencies to the App.
/// # Note
/// If you are using [Aery](https://crates.io/crates/aery), add it to the App before this plugin, or just add this plugin.
/// This plugin will add Aery if it's not in the app, since it is a unique plugin,
/// having multiple will panic.
pub struct TilesPlugin;

impl Plugin for TilesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {}
}
