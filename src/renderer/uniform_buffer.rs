use std::sync::Arc;
use std::marker::PhantomData;

pub struct UniformBuffer<UT> {
    pub(super) buffer: wgpu::Buffer,
    pub(super) unmapped: Arc<std::sync::atomic::AtomicBool>,
    phantom: PhantomData<UT>
}

impl<UT> UniformBuffer<UT> {
    pub fn new(device: &wgpu::Device) -> UniformBuffer<UT> {
        let uniform_buf_size = (core::mem::size_of::<UT>()) as u64;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: uniform_buf_size,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::MAP_WRITE
        });
        UniformBuffer {
            buffer,
            unmapped: Arc::new(std::sync::atomic::AtomicBool::new(true)),
            phantom: PhantomData
        }
    }
}