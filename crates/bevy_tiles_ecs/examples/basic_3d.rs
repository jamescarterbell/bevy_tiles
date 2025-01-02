use std::f32::consts::PI;

use bevy::{
    color::palettes::css::{BLUE, GREEN},
    pbr::{CascadeShadowConfig, CascadeShadowConfigBuilder},
    prelude::*,
    DefaultPlugins,
};
use bevy_tiles::{commands::TileCommandExt, coords::CoordIterator, tiles_3d::*, TilesPlugin};
use bevy_tiles_ecs::{
    commands::TileMapCommandsECSExt,
    tiles_3d::{TileCoord, TileEntityMapQuery},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilesPlugin)
        .add_systems(Startup, spawn)
        .add_systems(Update, move_character)
        .add_systems(PostUpdate, sync_tile_transforms)
        .run();
}

#[derive(Component, Clone)]
struct Block;

#[derive(Component)]
struct Character;

#[derive(Component)]
struct GameLayer;

fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube = meshes.add(Mesh::from(Cuboid {
        half_size: 0.5 * Vec3::ONE,
    }));

    let color_block = materials.add(StandardMaterial {
        base_color: BLUE.into(),
        ..default()
    });

    let color_player = materials.add(StandardMaterial {
        base_color: GREEN.into(),
        ..default()
    });

    commands.spawn((
        Camera3d::default(),
        Transform {
            translation: Vec3::new(0.0, 20.0, 20.0),
            ..Default::default()
        }
        .looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let mut tile_commands = commands.spawn_map(16);
    tile_commands.insert(GameLayer);

    // spawn a 10 * 10 room
    tile_commands.spawn_tile_batch(
        CoordIterator::new([-5, 0, 5], [5, 0, 5])
            .chain(CoordIterator::new([-5, 0, -5], [5, 0, -5]))
            .chain(CoordIterator::new([5, 0, -4], [5, 0, 4]))
            .chain(CoordIterator::new([-5, 0, -4], [-5, 0, 4])),
        (Block, Mesh3d(cube.clone()), MeshMaterial3d(color_block)),
    );

    // spawn a player
    tile_commands.spawn_tile(
        IVec3::ZERO,
        (
            Character,
            Mesh3d(cube.clone()),
            MeshMaterial3d(color_player),
        ),
    );

    // Spawn some light
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 10.0,
            ..default()
        }
        .build(),
    ));
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

    let y = keyboard_input.just_pressed(KeyCode::ShiftLeft) as i32
        - keyboard_input.just_pressed(KeyCode::ControlLeft) as i32;

    let z = keyboard_input.just_pressed(KeyCode::KeyS) as i32
        - keyboard_input.just_pressed(KeyCode::KeyW) as i32;

    let char_c = IVec3::from(*character.get_single().unwrap());
    let new_coord = [char_c[0] + x, char_c[1] + y, char_c[2] + z];

    if walls.get_at(new_coord).is_none() {
        commands
            .tile_map(map_id)
            .unwrap()
            .move_tile(char_c, new_coord);
    }
}

fn sync_tile_transforms(mut tiles: Query<(&TileCoord, &mut Transform), Changed<TileCoord>>) {
    for (tile_c, mut transform) in tiles.iter_mut() {
        transform.translation.x = tile_c[0] as f32;
        transform.translation.y = tile_c[1] as f32;
        transform.translation.z = tile_c[2] as f32;
    }
}
