use std::num::NonZeroU64;

use bevy::{
    ecs::{component::Component, world::FromWorld},
    log::debug,
    math::{Affine3, Vec2, Vec4},
    render::{
        render_resource::{
            BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutDescriptor,
            BindGroupLayoutEntry, BindingType, Buffer, BufferAddress, BufferBindingType,
            BufferDescriptor, BufferInitDescriptor, BufferUsages, CommandEncoder, ShaderSize,
            ShaderStages, ShaderType, StorageBuffer, UniformBuffer,
        },
        renderer::{RenderDevice, RenderQueue},
    },
    transform::components::GlobalTransform,
};

use crate::{
    buffer_helpers::*,
    chunk::{self, internal::ChunkUniforms},
    maps::internal::MapInfo,
};

#[derive(Component)]
pub struct ChunkBatchBindGroups {
    pub map_bind_group: BindGroup,
    pub chunk_bind_group: BindGroup,
}

/// Contains all the data for an individual chunk that can be
/// consolidated into the batch buffers.
#[derive(Component)]
pub struct ChunkBuffer {
    pub chunk_offset: Vec2,
    pub tile_instances: GpuStorageBuffer<u32>,
}

impl ChunkBuffer {
    pub fn new(chunk_uniforms: &mut ChunkUniforms) -> Self {
        Self {
            chunk_offset: Vec2::from(&chunk_uniforms.chunk_coord),
            tile_instances: GpuStorageBuffer::<u32>::from(
                chunk_uniforms
                    .tile_instances
                    .take()
                    .expect("Couldn't find TileInstances"),
            ),
        }
    }

    pub fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        self.tile_instances.write_buffer(device, queue);
    }
}

#[derive(Component)]
pub struct ChunkBatchBuffer {
    total_chunk_size: u64,
    batch_size: u64,
    pub chunk_offsets: GpuStorageBuffer<Vec2>,
    pub tile_instances: Buffer,
}

impl ChunkBatchBuffer {
    pub fn with_size_no_default_values(
        batch_size: usize,
        chunk_size: usize,
        device: &RenderDevice,
    ) -> Self {
        let total_chunk_size = chunk_size as u64 * chunk_size as u64;
        Self {
            total_chunk_size,
            batch_size: batch_size as u64,
            chunk_offsets: GpuStorageBuffer::<Vec2>::default(),
            tile_instances: device.create_buffer(&BufferDescriptor {
                label: None,
                size: total_chunk_size * batch_size as u64 * u32::SHADER_SIZE.get(),
                usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
                mapped_at_creation: false,
            }),
        }
    }

    /// # Note
    /// after call push, write_buffer needs to be called as well as using the commands
    /// from the command encoders to finish the copying.
    pub fn push(&mut self, command_encoder: &mut CommandEncoder, chunk_buffer: &ChunkBuffer) {
        let index = self.chunk_offsets.push(chunk_buffer.chunk_offset);
        command_encoder.copy_buffer_to_buffer(
            chunk_buffer.tile_instances.gpu_buffer().unwrap(),
            0,
            &self.tile_instances,
            index.get() as u64 * self.total_chunk_size * u32::SHADER_SIZE.get(),
            self.total_chunk_size * u32::SHADER_SIZE.get(),
        )
    }

    pub fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        self.chunk_offsets.write_buffer(device, queue);
    }

    pub fn bindings(&self) -> BindGroupEntries<2> {
        BindGroupEntries::with_indices((
            (0, self.chunk_offsets.binding().unwrap()),
            (1, self.tile_instances.as_entire_binding()),
        ))
    }

    pub fn layout_entries() -> Vec<BindGroupLayoutEntry> {
        vec![
            // off_sets
            GpuStorageBuffer::<Vec2>::binding_layout(0, ShaderStages::VERTEX_FRAGMENT),
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: Some(u32::min_size()),
                },
                count: None,
            },
        ]
    }
}

#[derive(Component)]
pub struct MapBatchBuffer {
    chunk_size: UniformBuffer<u32>,
    tile_size: UniformBuffer<f32>,
    grid_size: UniformBuffer<f32>,
    transform: UniformBuffer<MapTransformUniform>,
}

impl MapBatchBuffer {
    pub fn new(map_info: &MapInfo) -> Self {
        Self {
            chunk_size: map_info.chunk_size.into(),
            tile_size: map_info.tile_size.0.into(),
            grid_size: map_info.grid_size.0.into(),
            transform: MapTransformUniform::from(&map_info.transform).into(),
        }
    }

    pub fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        self.chunk_size.write_buffer(device, queue);
        self.transform.write_buffer(device, queue);
        self.tile_size.write_buffer(device, queue);
        self.grid_size.write_buffer(device, queue);
    }

    pub fn bindings(&self) -> BindGroupEntries<4> {
        BindGroupEntries::with_indices((
            (0, self.transform.binding().unwrap()),
            (1, self.chunk_size.binding().unwrap()),
            (2, self.tile_size.binding().unwrap()),
            (3, self.grid_size.binding().unwrap()),
        ))
    }

    pub fn layout_entries() -> Vec<BindGroupLayoutEntry> {
        vec![
            // transform
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(MapTransformUniform::SHADER_SIZE),
                },
                count: None,
            },
            // chunk_size
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(u32::SHADER_SIZE),
                },
                count: None,
            },
            // tile_size
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(f32::SHADER_SIZE),
                },
                count: None,
            },
            // grid_size
            BindGroupLayoutEntry {
                binding: 3,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(f32::SHADER_SIZE),
                },
                count: None,
            },
        ]
    }
}

#[derive(ShaderType, Clone, Default)]
pub struct MapTransformUniform {
    // Affine 4x3 matrix transposed to 3x4
    pub transform: [Vec4; 3],
    // 3x3 matrix packed in mat2x4 and f32 as:
    //   [0].xyz, [1].x,
    //   [1].yz, [2].xy
    //   [2].z
    pub inverse_transpose_model_a: [Vec4; 2],
    pub inverse_transpose_model_b: f32,
}

impl From<&GlobalTransform> for MapTransformUniform {
    fn from(value: &GlobalTransform) -> Self {
        let affine = Affine3::from(&value.affine());
        let (inverse_transpose_model_a, inverse_transpose_model_b) = affine.inverse_transpose_3x3();
        Self {
            transform: affine.to_transpose(),
            inverse_transpose_model_a,
            inverse_transpose_model_b,
        }
    }
}

pub struct ChunkBatchBindGroupLayouts {
    pub map_layouts: BindGroupLayout,
    pub chunk_layouts: BindGroupLayout,
}

impl FromWorld for ChunkBatchBindGroupLayouts {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let device = world
            .get_resource::<RenderDevice>()
            .expect("No render device found!");

        let map_layouts = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("bevy_tiles_map_bind_group"),
            entries: &MapBatchBuffer::layout_entries(),
        });

        let chunk_layouts = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("bevy_tiles_chunk_bind_group"),
            entries: &ChunkBatchBuffer::layout_entries(),
        });

        Self {
            map_layouts,
            chunk_layouts,
        }
    }
}
