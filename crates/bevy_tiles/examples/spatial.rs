use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::Vec2Swizzles,
    prelude::*,
    sprite::SpriteBundle,
    window::PrimaryWindow,
    DefaultPlugins,
};
use bevy_tiles::prelude::*;
use std::ops::{Deref, DerefMut};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilesPlugin)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_systems(Startup, spawn)
        .add_systems(Update, (add_damage, check_damage).chain())
        .add_systems(PostUpdate, sync_tile_transforms)
        .run();
}

#[derive(Component)]
struct Block;

#[derive(Component)]
struct Damage(usize);

impl Deref for Damage {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Damage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Component)]
struct GameLayer;

fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
    let block = asset_server.load("block.png");

    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 1.0,
            ..Camera2dBundle::default().projection
        },
        ..Default::default()
    });
    let mut tile_commands = commands.spawn_map::<2>(32, GameLayer);

    let sprite_bundle = SpriteBundle {
        texture: block,
        ..Default::default()
    };

    let size = 200;

    tile_commands.spawn_tile_batch(
        CoordIterator::new([-size, -size], [size, size]),
        move |_| (Block, sprite_bundle.clone()),
    );
}

fn add_damage(
    mut commands: Commands,
    mut block_maps: TileMapQuery<(Entity, Option<&mut Damage>), With<Block>>,
    map: Query<(Entity, &TileMap), With<GameLayer>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    let (map_id, map) = map.single();
    let (cam, cam_t) = camera.single();
    let mut blocks = block_maps.get_map_mut(map_id).unwrap();

    let cursor_pos = windows
        .single()
        .cursor_position()
        .and_then(|cursor| cam.viewport_to_world(cam_t, cursor.xy()))
        .map(|ray| ray.origin.truncate())
        .map(|pos| world_to_tile(pos.into(), 16.0));

    if let Some(damage_pos) = buttons
        .just_pressed(MouseButton::Left)
        .then_some(cursor_pos)
        .flatten()
    {
        let start = [damage_pos[0] - 2, damage_pos[1] - 2];
        let end = [damage_pos[0] + 2, damage_pos[1] + 2];
        for (block_id, damage) in blocks.iter_in_mut(start, end) {
            if let Some(mut damage) = damage {
                **damage += 1;
            } else {
                commands.entity(block_id).insert(Damage(1));
            }
        }
    }
    if let Some(damage_pos) = buttons
        .just_pressed(MouseButton::Right)
        .then_some(cursor_pos)
        .flatten()
    {
        let chunk_c = calculate_chunk_coordinate(damage_pos, map.get_chunk_size());
        commands.tile_map::<2>(map_id).despawn_chunk(chunk_c);
    }
}

fn check_damage(
    mut commands: Commands,
    mut damaged: Query<(Entity, &Damage, &mut Sprite), Changed<Damage>>,
) {
    for (id, damage, mut sprite) in damaged.iter_mut() {
        match **damage {
            x if x > 3 => commands.entity(id).despawn(),
            x => {
                let tint = 1.0 - x as f32 / 3.0;
                sprite.color = Color::Rgba {
                    red: 1.0,
                    green: tint,
                    blue: tint,
                    alpha: 1.0,
                }
            }
        }
    }
}

fn sync_tile_transforms(mut tiles: Query<(&TileCoord, &mut Transform), Changed<TileCoord>>) {
    for (tile_c, mut transform) in tiles.iter_mut() {
        transform.translation.x = tile_c[0] as f32 * 16.0;
        transform.translation.y = tile_c[1] as f32 * 16.0;
    }
}
