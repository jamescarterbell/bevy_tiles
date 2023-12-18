use std::iter::repeat_with;

use bevy::{
    app::{App, Startup, Update},
    core_pipeline::core_2d::Camera2dBundle,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    ecs::system::Commands,
    math::{Quat, Vec3},
    render::camera::OrthographicProjection,
    transform::{
        components::{GlobalTransform, Transform},
        TransformBundle,
    },
    DefaultPlugins,
};
use bevy_tiles::{
    commands::TileCommandExt, coords::CoordIterator, maps::TileMapLabel, TilesPlugin,
};
use bevy_tiles_render::{
    maps::{TileGridSize, TileMapRenderer, TileMapRenderingBundle, TileSize},
    TilesRenderPlugin,
};
use rand::Rng;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FrameTimeDiagnosticsPlugin,
            LogDiagnosticsPlugin::default(),
        ))
        .add_plugins((TilesPlugin, TilesRenderPlugin))
        .add_systems(Startup, spawn)
        .add_systems(Update, change_map)
        .run();
}

struct GameLayer;

impl TileMapLabel for GameLayer {
    const CHUNK_SIZE: usize = 128;
}

fn spawn(mut commands: Commands) {
    let mut tile_commands = commands.tiles::<GameLayer, 2>();
    tile_commands.spawn_map(TileMapRenderingBundle {
        tile_size: TileSize(16.0),
        grid_size: TileGridSize(18.0),
        tile_map_renderer: TileMapRenderer { batch_size: 512 },
        ..Default::default()
    });

    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 10.0,
            far: 1000.0,
            near: -1000.0,
            ..Default::default()
        },
        ..Default::default()
    });
}

fn change_map(mut commands: Commands) {
    let mut rng = rand::thread_rng();
    let changes = rng.gen_range(500..1000);
    let mut tile_commands = commands.tiles::<GameLayer, 2>();

    let spawn = rng.gen_bool(0.5);

    let tiles = Vec::from_iter(
        repeat_with(|| [rng.gen_range(-1000..1000), rng.gen_range(-1000..1000)]).take(changes),
    );

    if spawn {
        tile_commands.spawn_tile_batch(tiles, |_| ())
    } else {
        tile_commands.despawn_tile_batch(tiles)
    }
}
