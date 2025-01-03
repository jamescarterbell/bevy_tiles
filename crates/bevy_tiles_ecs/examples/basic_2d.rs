use bevy::{prelude::*, DefaultPlugins};
use bevy_tiles::{
    commands::TileCommandExt,
    coords::CoordIterator,
    maps::{TileDims, TileSpacing, UseTransforms},
    TilesPlugin,
};
use bevy_tiles_ecs::{
    commands::TileMapCommandsECSExt,
    tiles_2d::{TileCoord, TileEntityMapQuery},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilesPlugin)
        .add_systems(Startup, spawn)
        .add_systems(Update, move_character)
        .run();
}

#[derive(Component, Clone)]
struct Block;

#[derive(Component)]
struct Character;

#[derive(Component)]
struct GameLayer;

fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
    let block = asset_server.load("block.png");
    let character = asset_server.load("character.png");

    commands.spawn(Camera2d);
    let mut map = commands.spawn_map(16);
    map.insert((
        GameLayer,
        UseTransforms,
        TileDims([16.0, 16.0]),
        TileSpacing([4.0, 4.0]),
    ));

    let sprite = Sprite {
        image: block,
        ..Default::default()
    };

    // spawn a 10 * 10 room
    map.spawn_tile_batch(
        CoordIterator::new([-5, 5], [5, 5])
            .chain(CoordIterator::new([-5, -5], [5, -5]))
            .chain(CoordIterator::new([5, -4], [5, 4]))
            .chain(CoordIterator::new([-5, -4], [-5, 4])),
        (Block, sprite),
    );

    // spawn a player
    map.spawn_tile(
        IVec2::ZERO,
        (
            Character,
            Sprite {
                image: character,
                ..Default::default()
            },
        ),
    );
}

fn move_character(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    map: Query<Entity, With<GameLayer>>,
    character: Query<&TileCoord, With<Character>>,
    walls_maps: TileEntityMapQuery<(), With<Block>>,
) {
    let map_id = map.single();
    let walls = walls_maps.get_map(map_id).unwrap();

    let x = keyboard_input.just_pressed(KeyCode::KeyD) as i32
        - keyboard_input.just_pressed(KeyCode::KeyA) as i32;

    let y = keyboard_input.just_pressed(KeyCode::KeyW) as i32
        - keyboard_input.just_pressed(KeyCode::KeyS) as i32;

    let char_c = IVec2::from(*character.get_single().unwrap());
    let new_coord = char_c + IVec2::new(x, y);

    if (x != 0 || y != 0) && walls.get_at(new_coord).is_none() {
        commands
            .tile_map(map_id)
            .unwrap()
            .move_tile(char_c, new_coord);
    }
}
