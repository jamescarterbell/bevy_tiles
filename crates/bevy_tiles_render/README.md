# `bevy_tiles_render`

Rendering plugin for the bevy game engine, built on top of `bevy_tiles`.

# Features

Supports:
* Square Tilemaps.
* Isometric Tilemaps.
* Hex Tilemaps.
* Tile animation.

# How it Works

In `bevy_tiles`, each tile is automatically part of a chunk.  New and updated chunk information is copied over to the render world each frame.
All chunks being rendered have their information uploaded to the GPU, and multiple chunks are put together into batch draw calls.  No mesh is used
for these draw calls, instead the mesh is generated in the vertex shader, and discarded in the fragment shader if no tile exists there.

# Why use this over X?

The goal of the `bevy_tiles` project is to be the most erganomic and natural to use tile solution in the bevy game engine.
Querying for a tile or group of tiles should look as similar as possible to querying for a non tiled entity, and should 
look almost native to bevy from a code perspective.

# Why not use this over X?

This is a relatively new bundle of crates, and probably has some rough edges or missing features.  I (we, I'd love some help) will
get to a lot of features in the future, but it will take time and effort!