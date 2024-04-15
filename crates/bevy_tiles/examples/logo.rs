use bevy::{prelude::*, sprite::SpriteBundle, DefaultPlugins};
use bevy_tiles::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilesPlugin)
        .add_systems(Startup, (spawn, sync_tile_transforms).chain())
        .run();
}

#[derive(Component)]
struct Block;

#[derive(Component)]
struct GameLayer;

fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
    let block = asset_server.load("block.png");

    commands.spawn(Camera2dBundle {
        transform: Transform::from_translation(Vec3::new(480.0, 32.0, 0.0)),
        ..Default::default()
    });
    let mut map = commands.spawn_map::<2>(16, GameLayer);

    let sprite_bundle = SpriteBundle {
        texture: block,
        ..Default::default()
    };

    let logo = r#"
eeeee  eeee e    e e    e       eeee8 eeeee e     eeee  eeeee 
8   8  8    8    8 8    8         8     8   8     8     8     
8eee8e 8ee  88  e8 8eeee8         8     e   8     8ee   8eeee 
8    8 8     8  8    88           8     8   8     8         8 
88eee8 88ee  8ee8    88   eeeee   e   88eee 88eee 88ee  8ee88 "#;

    let logo = logo.split('\n').enumerate().flat_map(|(y, line)| {
        line.bytes().enumerate().filter_map(move |(x, byte)| {
            if byte == 56 || byte == 101 {
                Some([x as isize, 6 - y as isize])
            } else {
                None
            }
        })
    });

    // spawn a 10 * 10 room
    map.spawn_tile_batch(logo.collect::<Vec<[isize; 2]>>(), move |_| {
        (Block, sprite_bundle.clone())
    });
}

fn sync_tile_transforms(mut tiles: Query<(&TileCoord, &mut Transform), Changed<TileCoord>>) {
    for (tile_c, mut transform) in tiles.iter_mut() {
        transform.translation.x = tile_c[0] as f32 * 16.0;
        transform.translation.y = tile_c[1] as f32 * 16.0;
    }
}
