use crate::renderer::Renderer;
use std::marker::PhantomData;

pub struct Mesh<V: Copy + Clone> {
    pub(super) vertex_buf: wgpu::Buffer,
    pub(super) index_buf: wgpu::Buffer,
    pub(super) index_count: u64,
    phantom: PhantomData<V>
}

impl<V: Copy + Clone + 'static> Mesh<V> {
    pub fn new<UT>(vertices: Vec<V>, indices: Vec<u16>, renderer: &Renderer<UT, V>) -> Mesh<V> {
        let vertex_buf = renderer.device.create_buffer_mapped(
            vertices.len(),
            wgpu::BufferUsage::VERTEX
        ).fill_from_slice(&vertices);
        let index_buf = renderer.device.create_buffer_mapped(
            indices.len(),
            wgpu::BufferUsage::INDEX
        ).fill_from_slice(&indices);
        Mesh {
            vertex_buf,
            index_buf,
            index_count: indices.len() as u64,
            phantom: PhantomData
        }
    }
}