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

#[derive(Component)]
struct GameLayer;

fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
    let block = asset_server.load("block.png");
    let character = asset_server.load("character.png");

    commands.spawn(Camera2dBundle::default());
    let mut map = commands.spawn_map::<2>(16, GameLayer);

    let sprite_bundle = SpriteBundle {
        texture: block,
        ..Default::default()
    };

    // spawn a 10 * 10 room
    map.spawn_tile_batch(
        CoordIterator::new([-5, 5], [5, 5])
            .chain(CoordIterator::new([-5, -5], [5, -5]))
            .chain(CoordIterator::new([5, -4], [5, 4]))
            .chain(CoordIterator::new([-5, -4], [-5, 4])),
        move |_| (Block, sprite_bundle.clone()),
    );

    // spawn a player
    map.spawn_tile(
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
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    map: Query<Entity, With<GameLayer>>,
    character: Query<&TileCoord, With<Character>>,
    walls_maps: TileMapQuery<(), With<Block>>,
) {
    let map_id = map.single();
    let walls = walls_maps.get_map(map_id).unwrap();

    let x = keyboard_input.just_pressed(KeyCode::KeyD) as isize
        - keyboard_input.just_pressed(KeyCode::KeyA) as isize;

    let y = keyboard_input.just_pressed(KeyCode::KeyW) as isize
        - keyboard_input.just_pressed(KeyCode::KeyS) as isize;

    let char_c = character.get_single().unwrap();
    let new_coord = [char_c[0] + x, char_c[1] + y];

    if (x != 0 || y != 0) && walls.get_at(new_coord).is_none() {
        commands.move_tile(map_id, **char_c, new_coord);
    }
}

fn sync_tile_transforms(mut tiles: Query<(&TileCoord, &mut Transform), Changed<TileCoord>>) {
    for (tile_c, mut transform) in tiles.iter_mut() {
        transform.translation.x = tile_c[0] as f32 * 16.0;
        transform.translation.y = tile_c[1] as f32 * 16.0;
    }
}
