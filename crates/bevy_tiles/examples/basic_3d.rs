use std::f32::consts::PI;

use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*, DefaultPlugins};
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

fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));

    let color_block = materials.add(StandardMaterial {
        base_color: Color::BLUE,
        ..default()
    });

    let color_player = materials.add(StandardMaterial {
        base_color: Color::GREEN,
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform {
            translation: Vec3::new(0.0, 20.0, 20.0),
            ..Default::default()
        }
        .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    let block_mesh = PbrBundle {
        mesh: cube.clone(),
        material: color_block,
        ..Default::default()
    };

    let mut tile_commands = commands.tiles::<GameLayer, 3>();

    // spawn a 10 * 10 room
    tile_commands.spawn_tile_batch(
        CoordIterator::new([-5, 0, 5], [5, 0, 5])
            .chain(CoordIterator::new([-5, 0, -5], [5, 0, -5]))
            .chain(CoordIterator::new([5, 0, -4], [5, 0, 4]))
            .chain(CoordIterator::new([-5, 0, -4], [-5, 0, 4])),
        move |_| (Block, block_mesh.clone()),
    );

    // spawn a player
    tile_commands.spawn_tile(
        [0, 0, 0],
        (
            Character,
            PbrBundle {
                mesh: cube,
                material: color_player,
                ..Default::default()
            },
        ),
    );

    // Spawn some light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 10.0,
            ..default()
        }
        .into(),
        ..default()
    });
}

fn move_character(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    character: TileQuery<GameLayer, &TileCoord<3>, With<Character>, 3>,
    walls: TileQuery<GameLayer, (), With<Block>, 3>,
) {
    let mut tile_commands = commands.tiles::<GameLayer, 3>();

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

    let mut z = if keyboard_input.just_pressed(KeyCode::W) {
        -1
    } else {
        0
    };
    z += if keyboard_input.just_pressed(KeyCode::S) {
        1
    } else {
        0
    };

    let mut y = if keyboard_input.just_pressed(KeyCode::ShiftLeft) {
        1
    } else {
        0
    };
    y -= if keyboard_input.just_pressed(KeyCode::ControlLeft) {
        1
    } else {
        0
    };

    let char_c = character.get_single().unwrap();
    let new_coord = [char_c[0] + x, char_c[1] + y, char_c[2] + z];

    if walls.get_at(new_coord).is_none() {
        tile_commands.move_tile(**char_c, new_coord);
    }
}

fn sync_tile_transforms(
    // Important, you have to put the 3 in all these places at the moment!!!
    mut tiles: TileQuery<GameLayer, (&TileCoord<3>, &mut Transform), Changed<TileCoord<3>>, 3>,
) {
    for (tile_c, mut transform) in tiles.iter_mut() {
        transform.translation.x = tile_c[0] as f32;
        transform.translation.y = tile_c[1] as f32;
        transform.translation.z = tile_c[2] as f32;
    }
}
