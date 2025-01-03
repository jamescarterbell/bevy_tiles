# ![bevy_tiles](assets/logo.png)

[![Crates.io](https://img.shields.io/crates/v/bevy_tiles)](https://crates.io/crates/bevy_tiles)
[![docs](https://docs.rs/bevy_tiles/badge.svg)](https://docs.rs/bevy_tiles/)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/OxidizedGames/bevy_tiles/blob/main/LICENSE)
[![Crates.io](https://img.shields.io/crates/d/bevy_tiles)](https://crates.io/crates/bevy_tiles)

A general purpose grided library meant for adding chunked, gridded storages to bevy.  Since bevy does not expose a custom storage API, instead of being an actual entity storage, each chunk of a given map is treated as an entity, with the components for tiles on said maps stored inside of vectors on the entity.  For most cases, this works well as a tilemap storage solution, but if treating individual tiles as entities is desired, you can instead utilize [bevy_tiles_ecs](https://crates.io/crates/bevy_tiles_ecs) to handle the entity <-> mapping.  The goal is to keep the API surface as simple and intuitive as possible, and to avoid deferred operations/states where possible to make the structures more intuitive work with (ex: an update in one system should be seen by the following system, not the following frame when a system has run.). 

# Features

Currently, `bevy_tiles` supports the following:
* Automatic chunking (including access to chunk entities)
* Automatic map creation
* Hierarchical despawning of chunks and maps
* N-dimensional map support
* Map based quiries
* Spatial queries
* Batched operations for better performance on large groups of tiles or chunks
* Automagically handle hierarchical deletes.


# API

The basic API revolves around `TileQuery`'s, `TileCommands`, and `TileMapLabel`'s as seen below.

```rust
struct GameLayer;

impl TileMapLabel for GameLayer {
    const CHUNK_SIZE: usize = 16;
}

fn move_character(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    character: TileQuery<GameLayer, TileCoord, With<Character>>,
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

    let char_c = character.single();
    let new_coord = [char_c[0] + x, char_c[1] + y];

    if walls.get_at(new_coord).is_none() {
        tile_commands.move_tile(*char_c, new_coord);
    }
}
```

More examples can be found in the [examples](/examples) folder!


# Versions

| Bevy version  | bevy_tiles verison |
|---------------|--------------------|
| 0.15          | 0.2                |
| 0.12          | 0.1                |
| 0.11          | 0.1-dev            |