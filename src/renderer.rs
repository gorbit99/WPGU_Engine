pub mod mesh;
pub mod renderer_builder;
pub mod shader;
pub mod texture;
pub mod uniform_buffer;

use self::uniform_buffer::UniformBuffer;
use self::texture::Texture;
use self::mesh::Mesh;

use std::marker::PhantomData;

pub struct Renderer<UT, V> {
    pub(super) device: wgpu::Device,
    pub(super) queue: wgpu::Queue,
    pub(super) bind_group: wgpu::BindGroup,
    pub(super) bind_group_layout: wgpu::BindGroupLayout,
    pub(super) pipeline: wgpu::RenderPipeline,
    pub(super) swap_chain: wgpu::SwapChain,
    pub(super) uniform_buffer: UniformBuffer<UT>,
    pub(super) uniform_location: u32,
    pub(super) textures: Vec<(u32, wgpu::TextureView)>,
    pub(super) sampler: wgpu::Sampler,
    pub(super) sampler_location: u32,
    pub(super) phantom: PhantomData<V>
}

impl<UT, V: Copy> Renderer<UT, V> {
    pub fn render(&mut self, mesh: &Mesh<V>) {
        while !self.uniform_buffer.unmapped.load(std::sync::atomic::Ordering::SeqCst) {
            self.queue.submit(&[]);
        }

        let frame = self.swap_chain.get_next_texture();

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.view,
                        resolve_target: None,
                        load_op: wgpu::LoadOp::Clear,
                        store_op: wgpu::StoreOp::Store,
                        clear_color: wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0
                        }
                    }
                ],
                depth_stencil_attachment: None
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_index_buffer(&mesh.index_buf, 0);
            rpass.set_vertex_buffers(0, &[(&mesh.vertex_buf, 0)]);
            rpass.draw_indexed(0..mesh.index_count as u32, 0, 0..1);
        }
        self.queue.submit(&[encoder.finish()]);
    }

    pub fn fill_uniform_buffer(&mut self, value: &UT) {
        self.uniform_buffer.unmapped.store(false, std::sync::atomic::Ordering::SeqCst);
        let unmapped_clone = self.uniform_buffer.unmapped.clone();
        let data = unsafe {
            std::slice::from_raw_parts(
                (value as *const UT) as *const u8, std::mem::size_of::<UT>()
            )
        };
        self.uniform_buffer.buffer.map_write_async(
            0,
            core::mem::size_of::<UT>() as wgpu::BufferAddress,
            move |mapping_result| {
                let mapping = mapping_result.unwrap();
                mapping.data.copy_from_slice(data);
                unmapped_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            });
    }

    fn recreate_bind_group(&mut self) {
        let mut bindings = vec![
            wgpu::Binding {
                binding: self.uniform_location,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &self.uniform_buffer.buffer,
                    range: 0..core::mem::size_of::<UT>() as u64
                }
            }
        ];

        for texture_data in &self.textures {
            bindings.push(
                wgpu::Binding {
                    binding: texture_data.0,
                    resource: wgpu::BindingResource::TextureView(&texture_data.1)
                }
            )
        }

        bindings.push(
            wgpu::Binding {
                binding: self.sampler_location,
                resource: wgpu::BindingResource::Sampler(&self.sampler)
            }
        );

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            bindings: &bindings
        });
        self.bind_group = bind_group;
    }

    pub fn bind_texture(&mut self, location: u32, texture: &Texture) {
        for t in &mut self.textures {
            if t.0 == location {
                t.1 = texture.get_view();
                self.recreate_bind_group();
                return;
            }
        }
    }
}