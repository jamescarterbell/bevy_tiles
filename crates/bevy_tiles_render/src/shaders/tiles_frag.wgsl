
@group(1) @binding(1) var<uniform> chunk_size: u32;
@group(2) @binding(1) var<storage> tile_instances: array<u32>;

struct FragIn {
    @location(0) tile_index: u32,
    @location(1) chunk_index: u32,
};

/// Entry point for the fragment shader
@fragment
fn fs_main(in: FragIn) -> @location(0) vec4<f32> {
    let global_tile_index = in.chunk_index * chunk_size * chunk_size + in.tile_index;
    if tile_instances[global_tile_index] == 0u{
        discard;
    }
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}