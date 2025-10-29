use bevy::{
    color::palettes::tailwind::RED_400,
    prelude::*,
    sprite_render::{TileData, TilemapChunk, TilemapChunkTileData},
};
use bevy_tiles::{
    commands::TileCommandExt,
    maps::{TileDims, TileMap, UseTransforms},
    render::TilemapRenderingInfo,
    tiles_2d::TileMapQuery,
};
use rand::{Rng, RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, setup)
        .add_systems(Update, (update_tileset_image, update_tilemap))
        .run();
}

#[derive(Component, Deref, DerefMut)]
struct UpdateTimer(Timer);

#[derive(Resource, Deref, DerefMut)]
struct SeededRng(ChaCha8Rng);

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    // We're seeding the PRNG here to make this example deterministic for testing purposes.
    // This isn't strictly required in practical use unless you need your app to be deterministic.
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    let chunk_size = UVec2::splat(16);

    commands.spawn(Camera2d);
    let mut tile_commands = commands.spawn_map(32);
    tile_commands.insert((
        UseTransforms,
        TileDims([16.0, 16.0]),
        TilemapRenderingInfo {
            tile_display_size: UVec2 { x: 16, y: 16 },
            tileset: assets.load("tileset.png"),
            alpha_mode: bevy::sprite_render::AlphaMode2d::Blend,
        },
    ));

    let size = 100;

    for x in -32..32 {
        for y in -32..32 {
            tile_commands.insert_tile(
                IVec2::new(x, y),
                TileData {
                    tileset_index: 0,
                    ..Default::default()
                },
            );
        }
    }

    commands.insert_resource(SeededRng(rng));
}

fn update_tileset_image(
    chunk_query: Query<&TilemapRenderingInfo>,
    mut events: MessageReader<AssetEvent<Image>>,
    mut images: ResMut<Assets<Image>>,
) {
    if let Ok(chunk) = chunk_query.single() {
        for event in events.read() {
            if event.is_loaded_with_dependencies(chunk.tileset.id()) {
                let image = images.get_mut(&chunk.tileset).unwrap();
                image.reinterpret_stacked_2d_as_array(4);
            }
        }
    }
}

fn update_tilemap(
    map: Single<Entity, With<TileMap>>,
    mut tile_query: TileMapQuery<(&mut TileData)>,
    mut rng: ResMut<SeededRng>,
) {
    let x = (rng.0.next_u32() % 64) as i32 - 32;
    let y = (rng.0.next_u32() % 64) as i32 - 32;

    let map = *map;
    let mut map = tile_query.get_map_mut(map).unwrap();
    let data = map.get_at_mut([x, y]).unwrap();

    data.tileset_index = (data.tileset_index + 1) % 4
}

// // find the data for an arbitrary tile in the chunk and log its data
// fn log_tile(tilemap: Single<(&TilemapChunk, &TilemapChunkTileData)>, mut local: Local<u16>) {
//     let (chunk, data) = tilemap.into_inner();
//     let Some(tile_data) = data.tile_data_from_tile_pos(chunk.chunk_size, UVec2::new(3, 4)) else {
//         return;
//     };
//     // log when the tile changes
//     if tile_data.tileset_index != *local {
//         info!(?tile_data, "tile_data changed");
//         *local = tile_data.tileset_index;
//     }
// }
