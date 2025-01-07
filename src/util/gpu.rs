use std::sync::Arc;
use wgpu::{Buffer, BufferDescriptor, BufferUsages, Device};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferUsage {
    Vertex,
    Index,
    Uniform,
    Storage,
    Indirect,
}

impl From<BufferUsage> for BufferUsages {
    fn from(usage: BufferUsage) -> Self {
        match usage {
            BufferUsage::Vertex => BufferUsages::VERTEX,
            BufferUsage::Index => BufferUsages::INDEX,
            BufferUsage::Uniform => BufferUsages::UNIFORM,
            BufferUsage::Storage => BufferUsages::STORAGE,
            BufferUsage::Indirect => BufferUsages::INDIRECT,
        }
    }
}

pub struct GpuBuffer {
    buffer: Arc<Buffer>,
    size: u64,
    usage: BufferUsage,
}

impl GpuBuffer {
    pub fn new(device: &Device, data: &[u8], usage: BufferUsage) -> Self {
        let buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: data.len() as u64,
            usage: usage.into(),
            mapped_at_creation: true,
        });

        // Copy data to buffer
        buffer.slice(..).get_mapped_range_mut().copy_from_slice(data);
        buffer.unmap();

        Self {
            buffer: Arc::new(buffer),
            size: data.len() as u64,
            usage,
        }
    }

    pub fn new_empty(device: &Device, size: u64, usage: BufferUsage) -> Self {
        let buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size,
            usage: usage.into(),
            mapped_at_creation: false,
        });

        Self {
            buffer: Arc::new(buffer),
            size,
            usage,
        }
    }

    pub fn update(&self, device: &Device, queue: &wgpu::Queue, data: &[u8], offset: u64) {
        assert!(offset + data.len() as u64 <= self.size, "Buffer update out of bounds");
        queue.write_buffer(&self.buffer, offset, data);
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn usage(&self) -> BufferUsage {
        self.usage
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
} 