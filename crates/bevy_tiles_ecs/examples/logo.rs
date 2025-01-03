use bevy::{prelude::*, DefaultPlugins};
use bevy_tiles::{
    commands::TileCommandExt,
    maps::{TileDims, TileSpacing, UseTransforms},
    TilesPlugin,
};
use bevy_tiles_ecs::commands::TileMapCommandsECSExt;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilesPlugin)
        .add_systems(Startup, spawn)
        .run();
}

#[derive(Component, Clone)]
struct Block;

#[derive(Component)]
struct GameLayer;

fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
    let block = asset_server.load("block.png");

    commands.spawn((
        Camera2d,
        Transform::from_translation(Vec3::new(480.0, 32.0, 0.0)),
    ));
    let mut map = commands.spawn_map(16);
    map.insert((
        GameLayer,
        UseTransforms,
        TileDims([16.0, 16.0]),
        TileSpacing([0.0, 0.0]),
    ));

    let logo = r#"
eeeee  eeee e    e e    e       eeee8 eeeee e     eeee  eeeee 
8   8  8    8    8 8    8         8     8   8     8     8     
8eee8e 8ee  88  e8 8eeee8         8     e   8     8ee   8eeee 
8    8 8     8  8    88           8     8   8     8         8 
88eee8 88ee  8ee8    88   eeeee   e   88eee 88eee 88ee  8ee88 "#;

    let logo = logo.split('\n').enumerate().flat_map(|(y, line)| {
        line.bytes().enumerate().filter_map(move |(x, byte)| {
            if byte == 56 || byte == 101 {
                Some([x as i32, 6 - y as i32])
            } else {
                None
            }
        })
    });

    // spawn a 10 * 10 room
    map.spawn_tile_batch(
        logo.collect::<Vec<[i32; 2]>>(),
        (
            Block,
            Sprite {
                image: block,
                ..Default::default()
            },
        ),
    );
}
