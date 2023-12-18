use std::{marker::PhantomData, mem};

use bevy::{
    ecs::{component::Component, system::Resource, world::FromWorld},
    render::{
        render_resource::{
            BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBindingType,
            BufferUsages, GpuArrayBufferable, ShaderStages, StorageBuffer,
        },
        renderer::{RenderDevice, RenderQueue},
    },
    utils::nonmax::NonMaxU32,
};

/// Stores an array of elements to be transferred to the GPU and made accessible to shaders as a read-only array.
/// This is modified from bevy's GpuArrayBuffer
pub struct GpuStorageBuffer<T: GpuArrayBufferable> {
    gpu_buffer: StorageBuffer<Vec<T>>,
    buffer: Vec<T>,
}

impl<T: GpuArrayBufferable> From<Vec<T>> for GpuStorageBuffer<T> {
    fn from(value: Vec<T>) -> Self {
        let mut gpu_buffer: StorageBuffer<Vec<T>> = Default::default();
        gpu_buffer.add_usages(BufferUsages::COPY_SRC);
        Self {
            gpu_buffer,
            buffer: value,
        }
    }
}

impl<T: GpuArrayBufferable> Default for GpuStorageBuffer<T> {
    fn default() -> Self {
        let mut gpu_buffer: StorageBuffer<Vec<T>> = Default::default();
        gpu_buffer.add_usages(BufferUsages::COPY_SRC);
        Self {
            gpu_buffer,
            buffer: Default::default(),
        }
    }
}

impl<T: GpuArrayBufferable> GpuStorageBuffer<T> {
    pub fn clear(&mut self) {
        self.buffer.clear()
    }

    pub fn gpu_buffer(&self) -> Option<&Buffer> {
        self.gpu_buffer.buffer()
    }

    pub fn push(&mut self, value: T) -> NonMaxU32 {
        let index = NonMaxU32::new(self.buffer.len() as u32).unwrap();
        self.buffer.push(value);
        index
    }

    pub fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        self.gpu_buffer.set(mem::take(&mut self.buffer));
        self.gpu_buffer.write_buffer(device, queue);
    }

    pub fn binding_layout(binding: u32, visibility: ShaderStages) -> BindGroupLayoutEntry {
        BindGroupLayoutEntry {
            binding,
            visibility,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: Some(T::min_size()),
            },
            count: None,
        }
    }

    pub fn binding(&self) -> Option<BindingResource> {
        self.gpu_buffer.binding()
    }
    pub fn insert(&mut self, index: usize, value: T) {
        *self.buffer.get_mut(index).unwrap() = value;
    }

    // Fails if used on the wrong size buffer
    /// SAFETY: Use carefully, this is to allow for parallel writes to sections of a buffer.
    pub unsafe fn raw_insert(&self, index: usize, value: T) {
        let spot: *const T = self.buffer.get(index).unwrap();
        let spot: *mut T = spot as *mut T;
        *spot = value;
    }
}

impl<T: GpuArrayBufferable> GpuStorageBuffer<T>
where
    T: Default,
{
    /// Creates a buffer of the given size filled with default values
    pub fn with_size(size: usize) -> Self {
        Self {
            buffer: vec![T::default(); size],
            gpu_buffer: Default::default(),
        }
    }
}
