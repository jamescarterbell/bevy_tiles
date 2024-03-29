use bevy::{prelude::*, sprite::SpriteBundle, DefaultPlugins};
use bevy_tiles::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilesPlugin)
        .add_systems(Startup, spawn)
        .add_systems(Update, move_character)
        .add_systems(PostUpdate, sync_tile_transforms)
        .run();
}

#[derive(Component)]
struct Block;

#[derive(Component)]
struct Character;

struct GameLayer;

impl TileMapLabel for GameLayer {
    const CHUNK_SIZE: usize = 16;
}

fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
    let block = asset_server.load("block.png");
    let character = asset_server.load("character.png");

    commands.spawn(Camera2dBundle::default());
    let mut tile_commands = commands.tiles::<GameLayer, 2>();

    let sprite_bundle = SpriteBundle {
        texture: block,
        ..Default::default()
    };

    // spawn a 10 * 10 room
    tile_commands.spawn_tile_batch(
        CoordIterator::new([-5, 5], [5, 5])
            .chain(CoordIterator::new([-5, -5], [5, -5]))
            .chain(CoordIterator::new([5, -4], [5, 4]))
            .chain(CoordIterator::new([-5, -4], [-5, 4])),
        move |_| (Block, sprite_bundle.clone()),
    );

    // spawn a player
    tile_commands.spawn_tile(
        [0, 0],
        (
            Character,
            SpriteBundle {
                texture: character,
                ..Default::default()
            },
        ),
    );
}

fn move_character(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    character: TileQuery<GameLayer, &TileCoord, With<Character>>,
    walls: TileQuery<GameLayer, (), With<Block>>,
) {
    let mut tile_commands = commands.tiles::<GameLayer, 2>();

    let mut x = if keyboard_input.just_pressed(KeyCode::A) {
        -1
    } else {
        0
    };
    x += if keyboard_input.just_pressed(KeyCode::D) {
        1
    } else {
        0
    };

    let mut y = if keyboard_input.just_pressed(KeyCode::W) {
        1
    } else {
        0
    };

    y -= if keyboard_input.just_pressed(KeyCode::S) {
        1
    } else {
        0
    };

    let char_c = character.get_single().unwrap();
    let new_coord = [char_c[0] + x, char_c[1] + y];

    if (x != 0 || y != 0) && walls.get_at(new_coord).is_none() {
        tile_commands.move_tile(**char_c, new_coord);
    }
}

fn sync_tile_transforms(
    mut tiles: TileQuery<GameLayer, (&TileCoord, &mut Transform), Changed<TileCoord>>,
) {
    for (tile_c, mut transform) in tiles.iter_mut() {
        transform.translation.x = tile_c[0] as f32 * 16.0;
        transform.translation.y = tile_c[1] as f32 * 16.0;
    }
}
