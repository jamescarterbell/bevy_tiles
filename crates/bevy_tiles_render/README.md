# `bevy_tiles_render`

Rendering plugin for the bevy game engine, built on top of `bevy_tiles`.

# Features

Supports:
* Square Tilemaps.
* Isometric Tilemaps.
* Hex Tilemaps.
* Tile animation.

# How it Works

In `bevy_tiles`, each tile is automatically part of a chunk.  Each frame, if a tile in a chunk has changed,
we can regenerate the render mesh for all the tiles in a chunk, stick it on the chunk, then send it to the render app.
This allows for more efficient rendering, and has the added benefit of letting you remove a chunk from the map, but keeping
the render mesh, allowing for higher performance in large static chunk scenarios.

# Why use this over X?

The goal of the `bevy_tiles` project is to be the most erganomic and natural to use tile solution in the bevy game engine.
Querying for a tile or group of tiles should look as similar as possible to querying for a non tiled entity, and should 
look almost native to bevy from a code perspective.

# Why not use this over X?

This is a relatively new bundle of crates, and probably has some rough edges or missing features.  I (we, I'd love some help) will
get to a lot of features in the future, but it will take time and effort!