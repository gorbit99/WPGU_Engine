use std::path::Path;
use image::GenericImageView;
use crate::renderer::Renderer;

pub struct Texture {
    texture: wgpu::Texture,
    texture_extent: wgpu::Extent3d
}

impl Texture {
    pub(super) fn new_from_data(width: u32, height: u32, data: &Vec<u8>, device: &mut wgpu::Device, queue: &mut wgpu::Queue) -> Texture {
        let texture_extent = wgpu::Extent3d {
            width,
            height,
            depth: 1
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST
        });
        let temp_buf = device.create_buffer_mapped(data.len(), wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(data.as_slice());

        let mut init_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            todo: 0
        });
        init_encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &temp_buf,
                offset: 0,
                row_pitch: 4 * width,
                image_height: height
            },
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0
                }
            },
            texture_extent
        );
        queue.submit(&[init_encoder.finish()]);

        Texture {
            texture,
            texture_extent
        }
    }

    pub fn new<UT, V>(path: &Path, renderer: &mut Renderer<UT, V>) -> Texture {
        let (w, h, data) = {
            let img = image::open(&path).unwrap();
            let w = img.width();
            let h = img.height();
            (w, h, img.into_rgba().clone().into_raw())
        };
        Self::new_from_data(w, h, &data, &mut renderer.device, &mut renderer.queue)
    }

    pub fn get_view(&self) -> wgpu::TextureView {
        self.texture.create_default_view()
    }
}