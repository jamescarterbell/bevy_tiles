//! Components that can affect map rendering

use bevy::{
    ecs::{bundle::Bundle, component::Component},
    prelude::Deref,
    transform::components::{GlobalTransform, Transform},
};

pub(crate) mod internal;

#[derive(Bundle, Default)]
pub struct TileMapRenderingBundle {
    pub tile_map_renderer: TileMapRenderer,
    pub tile_size: TileSize,
    pub grid_size: TileGridSize,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

/// Marks a tilemap as renderable, without this it cannot be rendered.
#[derive(Clone, Component)]
pub struct TileMapRenderer {
    pub batch_size: u32,
}

impl Default for TileMapRenderer {
    fn default() -> Self {
        Self { batch_size: 128 }
    }
}

/// The size of a tile in pixels.
#[derive(Clone, Deref, Component)]
pub struct TileSize(pub f32);

/// Defaults to 16 pixels
impl Default for TileSize {
    fn default() -> Self {
        Self(16.0)
    }
}
/// The size of a tile grid in pixels.
/// # Example
/// A [`TileSize`] of 16 with a [`GridSize`] of 18 would lead to a 2 pixel gap between tiles.
/// A [`TileSize`] of 16 with a [`GridSize`] of 14 would lead to a 2 pixel overlap between tiles.
#[derive(Clone, Deref, Component)]
pub struct TileGridSize(pub f32);

/// Defaults to 16 pixels
impl Default for TileGridSize {
    fn default() -> Self {
        Self(16.0)
    }
}
