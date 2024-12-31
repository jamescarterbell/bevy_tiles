use bevy::{prelude::*, DefaultPlugins};
use bevy_tiles::{commands::TileCommandExt, coords::CoordIterator, tiles_2d::*, TilesPlugin};
use bevy_tiles_ecs::{commands::TileMapCommandsECSExt, tiles_2d::TileEntityMapQuery};

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

    commands.spawn(Camera2d);
    let mut map = commands.spawn_map(16);
    map.insert(GameLayer);

    let sprite = Sprite {
        image: block,
        ..Default::default()
    };

    // spawn a 10 * 10 room
    // map.spawn_tile_batch(
    //     CoordIterator::new([-5, 5], [5, 5])
    //         .chain(CoordIterator::new([-5, -5], [5, -5]))
    //         .chain(CoordIterator::new([5, -4], [5, 4]))
    //         .chain(CoordIterator::new([-5, -4], [-5, 4])),
    //     move |_| (Block, sprite_bundle.clone()),
    // );

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

fn sync_tile_transforms(mut tiles: Query<(&TileCoord, &mut Transform), Changed<TileCoord>>) {
    for (tile_c, mut transform) in tiles.iter_mut() {
        transform.translation = Vec3::from((Vec2::from(*tile_c) * Vec2::ONE * 16.0, 0.0));
    }
}
