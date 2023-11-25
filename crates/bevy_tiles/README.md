# ![bevy_tiles](assets/logo.png)

[![Crates.io](https://img.shields.io/crates/v/bevy_tiles)](https://crates.io/crates/bevy_tiles)
[![docs](https://docs.rs/bevy_tiles/badge.svg)](https://docs.rs/bevy_tiles/)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/OxidizedGames/bevy_tiles/blob/main/LICENSE)
[![Crates.io](https://img.shields.io/crates/d/bevy_tiles)](https://crates.io/crates/bevy_tiles)

A general purpose grided entity library meant to support tilemap libraries, or other libraries that require accessing entities in a grid based manner built on top of the [`aery`](https://github.com/iiYese/aery) relations crate.  The goal is to keep the API surface as simple and intuitive as possible, and to avoid deferred operations/states where possible to make the structures more intuitive work with (ex: an update in one system should be seen by the following system, not the following frame.). 

# Features

Currently, `bevy_tiles` supports the following:
* Automatic chunking (including access to chunk entities)
* Automatic map creation
* Hierarchical despawning of chunks and maps
* N-dimensional map support
* Map based quiries
* Spatial queries
* Batched operations for better performance on large groups of tiles or chunks

Upcoming features:
* Automatigically handle hierarchical deletes (via aery support or supported directly in this crate)
* Sort tiles in memory based on chunk and map (will require bevy API additions in the future)

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

| Bevy version | bevy_tiles verison |
|--------------|--------------------|
| 0.12         | 0.1                |
| 0.11         | 0.1-dev            |