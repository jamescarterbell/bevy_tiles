#define_import_path bevy_tiles::vert
// Import the standard 2d mesh uniforms and set their bind groups
#import bevy_render::view::View
#import bevy_render::globals::Globals
#import bevy_render::{
    instance_index::get_instance_index,
    maths::{affine_to_square, mat2x4_f32_to_mat3x3_unpack},
}

@group(0) @binding(0) var<uniform> view: View;

@group(0) @binding(1) var<uniform> globals: Globals;

// Map level uniforms
@group(1) @binding(0) var<uniform> mesh: Mesh2d;
@group(1) @binding(1) var<uniform> chunk_size: u32;
@group(1) @binding(2) var<uniform> tile_size: f32;
@group(1) @binding(3) var<uniform> grid_size: f32;

// Chunk level uniforms
@group(2) @binding(0) var<storage> chunk_offsets: array<vec2<f32>>;

// The structure of the vertex buffer is as specified in `specialize()`
struct VertIn {
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
};

struct VertOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tile_index: u32,
    @location(1) chunk_index: u32,
};

// LUT for quad verts
var<private> positions: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(0.0, 0.0),
);

/// Entry point for the vertex shader
@vertex
fn vs_main(v: VertIn) -> VertOut {
    let tile_index = v.vertex_index / 6u;
    let chunk_index = v.instance_index;
    let tile_offset = vec2<f32>(
        f32(tile_index % chunk_size),
        f32((tile_index / chunk_size) % chunk_size)
    );
    let grid_offset = grid_size - tile_size;
    let chunk_offset = f32(chunk_size) * chunk_offsets[chunk_index];
    let vertex_position = 
        // Base tile position
        tile_size * (positions[v.vertex_index % 6u] + tile_offset + chunk_offset) +
        // Account for grid size
        (grid_offset) * (tile_offset + chunk_offset) + vec2<f32>(grid_offset / 2.0);

    let model = affine_to_square(mesh.model);
    let clip_position = mesh2d_position_local_to_clip(model, vec4<f32>(vertex_position, 1.0, 1.0));

    var out: VertOut;
    out.clip_position = clip_position;
    out.tile_index = tile_index;
    out.chunk_index = chunk_index;
    return out;
}

struct Mesh2d {
    model: mat3x4<f32>,
    inverse_transpose_model_a: mat2x4<f32>,
    inverse_transpose_model_b: f32,
    flags: u32,
};


fn mesh2d_position_local_to_world(model: mat4x4<f32>, vertex_position: vec4<f32>) -> vec4<f32> {
    return model * vertex_position;
}

fn mesh2d_position_world_to_clip(world_position: vec4<f32>) -> vec4<f32> {
    return view.view_proj * world_position;
}

// NOTE: The intermediate world_position assignment is important
// for precision purposes when using the 'equals' depth comparison
// function.
fn mesh2d_position_local_to_clip(model: mat4x4<f32>, vertex_position: vec4<f32>) -> vec4<f32> {
    let world_position = mesh2d_position_local_to_world(model, vertex_position);
    return mesh2d_position_world_to_clip(world_position);
}

fn mesh2d_tangent_local_to_world(model: mat4x4<f32>, vertex_tangent: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(
        mat3x3<f32>(
            model[0].xyz,
            model[1].xyz,
            model[2].xyz
        ) * vertex_tangent.xyz,
        vertex_tangent.w
    );
}