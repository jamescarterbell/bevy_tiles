use bevy::{
    color::palettes::tailwind::RED_400,
    prelude::*,
    sprite_render::{TileData, TilemapChunk, TilemapChunkTileData},
};
use bevy_tiles::{commands::TileCommandExt, maps::{TileDims, UseTransforms}, render::TilemapRenderingInfo};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (update_tileset_image, update_tilemap),
        )
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

    let chunk_size = UVec2::splat(64);

    commands.spawn(Camera2d);
    let mut tile_commands = commands.spawn_map(32);
    tile_commands.insert((
        UseTransforms,
        TileDims([16.0, 16.0]),
        TilemapRenderingInfo {
            tile_display_size: UVec2 { x: 16, y: 16 },
            tileset: assets.load("tileset.png"),
            alpha_mode: bevy::sprite_render::AlphaMode2d::Blend,
        }
    ));

    let size = 100;

    for i in -64..64 {
        tile_commands.insert_tile(IVec2::new(i, i), TileData {tileset_index: 0, ..Default::default()});
    }
    for i in -64..64 {
        tile_commands.insert_tile(IVec2::new(0, i), TileData {tileset_index: 0, ..Default::default()});
    }

    commands.insert_resource(SeededRng(rng));
}

// #[derive(Component)]
// struct MovePlayer;

// fn spawn_fake_player(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<ColorMaterial>>,
//     chunk: Single<&TilemapChunk>,
// ) {
//     let mut transform = chunk.calculate_tile_transform(UVec2::new(0, 0));
//     transform.translation.z = 1.;

//     commands.spawn((
//         Mesh2d(meshes.add(Rectangle::new(8., 8.))),
//         MeshMaterial2d(materials.add(Color::from(RED_400))),
//         transform,
//         MovePlayer,
//     ));

//     let mut transform = chunk.calculate_tile_transform(UVec2::new(5, 6));
//     transform.translation.z = 1.;

//     // second "player" to visually test a non-zero position
//     commands.spawn((
//         Mesh2d(meshes.add(Rectangle::new(8., 8.))),
//         MeshMaterial2d(materials.add(Color::from(RED_400))),
//         transform,
//     ));
// }

// fn move_player(
//     mut player: Single<&mut Transform, With<MovePlayer>>,
//     time: Res<Time>,
//     chunk: Single<&TilemapChunk>,
// ) {
//     let t = (ops::sin(time.elapsed_secs()) + 1.) / 2.;

//     let origin = chunk
//         .calculate_tile_transform(UVec2::new(0, 0))
//         .translation
//         .x;
//     let destination = chunk
//         .calculate_tile_transform(UVec2::new(63, 0))
//         .translation
//         .x;

//     player.translation.x = origin.lerp(destination, t);
// }

fn update_tileset_image(
    chunk_query: Query<&TilemapRenderingInfo>,
    mut events: MessageReader<AssetEvent<Image>>,
    mut images: ResMut<Assets<Image>>,
) {
    if let Ok(chunk) = chunk_query.single(){
        for event in events.read() {
            if event.is_loaded_with_dependencies(chunk.tileset.id()) {
                let image = images.get_mut(&chunk.tileset).unwrap();
                image.reinterpret_stacked_2d_as_array(2);
            }
        }
    }
}

fn update_tilemap(
    mut query: Query<(&TilemapChunkTileData, &Transform)>,
    mut rng: ResMut<SeededRng>,
) {
    for (tile_data, transform) in query.iter_mut() {
        //println!("{:?}", transform);
    }
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
