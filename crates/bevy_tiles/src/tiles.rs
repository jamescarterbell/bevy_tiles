use bevy::prelude::*;

mod tile_query;

pub use tile_query::*;

/// The index of a tile in a given chunk.
/// # Note:
/// It probably won't break anything to manually copy this
/// to put it on your own entities, but this is only accurate
/// when mutated by the plugin.
#[derive(Component, Clone, Copy, PartialEq, Eq, Deref, Debug)]
pub struct TileIndex(pub(crate) usize);

/// The coordinate of a tile in a given map.
/// # Note:
/// It probably won't break anything to manually copy this
/// to put it on your own entities, but this is only accurate
/// when mutated by the plugin.
#[derive(Component, Deref, Clone, Copy, PartialEq, Eq, Debug)]
pub struct TileCoord<const N: usize>(pub(crate) [i32; N]);

impl From<TileCoord<3>> for IVec3 {
    fn from(value: TileCoord<3>) -> Self {
        value.0.into()
    }
}

impl From<TileCoord<2>> for IVec2 {
    fn from(value: TileCoord<2>) -> Self {
        value.0.into()
    }
}

impl From<TileCoord<3>> for Vec3 {
    fn from(value: TileCoord<3>) -> Self {
        Vec3::new(value[0] as f32, value[1] as f32, value[2] as f32)
    }
}

impl From<TileCoord<2>> for Vec2 {
    fn from(value: TileCoord<2>) -> Self {
        Vec2::new(value[0] as f32, value[1] as f32)
    }
}

/// A relation on tiles that point towards the chunk they are a part of.
#[derive(Component, Deref, Debug)]
pub struct InChunk(pub(crate) Entity);
