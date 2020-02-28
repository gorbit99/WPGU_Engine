use std::marker::PhantomData;

use wgpu::VertexFormat;

use crate::renderer::shader::Shader;
use crate::renderer::uniform_buffer::UniformBuffer;

use super::Renderer;
use super::texture::Texture;

pub struct RendererBuilder<'a> {
    window: &'a winit::window::Window,
    vs: Option<&'a Shader>,
    fs: Option<&'a Shader>,
    uniform_location: u32,
    textures: Vec<u32>,
    culling: (wgpu::FrontFace, wgpu::CullMode),
    vertex_attributes: Vec<(u32, wgpu::VertexFormat)>,
    sampler_location: u32
}

impl<'a> RendererBuilder<'a> {
    pub fn new(window: &winit::window::Window) -> RendererBuilder {
        RendererBuilder {
            window,
            vs: None,
            fs: None,
            uniform_location: 0,
            textures: Vec::new(),
            culling: (wgpu::FrontFace::Ccw, wgpu::CullMode::None),
            vertex_attributes: Vec::new(),
            sampler_location: 1
        }
    }

    pub fn add_vertex_shader(mut self, vs: &'a Shader) -> RendererBuilder<'a> {
        self.vs = Some(vs);
        self
    }

    pub fn add_fragment_shader(mut self, fs: &'a Shader) -> RendererBuilder<'a> {
        self.fs = Some(fs);
        self
    }

    pub fn add_texture(mut self, location: u32) -> RendererBuilder<'a> {
        self.textures.push(location);
        self
    }

    pub fn set_sampler_location(mut self, sampler_location: u32) -> RendererBuilder<'a> {
        self.sampler_location = sampler_location;
        self
    }

    pub fn set_culling(mut self, front_face: wgpu::FrontFace, cull_mode: wgpu::CullMode) -> RendererBuilder<'a> {
        self.culling = (front_face, cull_mode);
        self
    }

    pub fn add_vertex_attribute(mut self, location: u32, format: wgpu::VertexFormat) -> RendererBuilder<'a> {
        self.vertex_attributes.push((location, format));
        self
    }

    pub fn set_uniform_location(mut self, location: u32) -> RendererBuilder<'a> {
        self.uniform_location = location;
        self
    }

    pub fn build<UT, V>(mut self) -> Result<Renderer<UT, V>, &'static str> {
        let size = self.window.inner_size();
        let surface = wgpu::Surface::create(self.window);

        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                backends: wgpu::BackendBit::PRIMARY
            }
        ).unwrap();

        let (mut device, mut queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false
                },
                limits: wgpu::Limits::default()
            }
        );

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Vsync
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let mut bindings = vec![
            wgpu::BindGroupLayoutBinding {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false }
            }
        ];
        for texture_data in &self.textures {
            bindings.push(
                wgpu::BindGroupLayoutBinding {
                    binding: *texture_data,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2
                    }
                }
            )
        }
        bindings.push(
            wgpu::BindGroupLayoutBinding {
                binding: self.sampler_location,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Sampler
            }
        );

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &bindings
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout]
        });

        let uniform_buffer = UniformBuffer::new(&device);

        let mut default_texture_data = Vec::new();
        for x in 0..256 {
            for y in 0..256 {
                if (x < 128) == (y < 128) {
                    default_texture_data.push(0);
                    default_texture_data.push(0);
                    default_texture_data.push(0);
                    default_texture_data.push(255);
                } else {
                    default_texture_data.push(255);
                    default_texture_data.push(0);
                    default_texture_data.push(255);
                    default_texture_data.push(255);
                }
            }
        }

        let mut bindings = vec![
            wgpu::Binding {
                binding: self.uniform_location,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &uniform_buffer.buffer,
                    range: 0..core::mem::size_of::<UT>() as u64
                }
            }
        ];

        let mut textures = Vec::new();
        let default_texture = Texture::new_from_data(256, 256, &default_texture_data, &mut device, &mut queue);

        for texture_data in &self.textures {
            textures.push(
                (*texture_data, default_texture.get_view())
            );
        }

        for texture in &textures {
            bindings.push(
                wgpu::Binding {
                    binding: texture.0,
                    resource: wgpu::BindingResource::TextureView(&texture.1)
                }
            );
        }

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare_function: wgpu::CompareFunction::Always
        });

        bindings.push(
            wgpu::Binding {
                binding: self.sampler_location,
                resource: wgpu::BindingResource::Sampler(&sampler)
            }
        );

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &bindings
        });

        if self.vs.is_none() {
            return Err("Vertex stage was not specified!");
        }
        if self.fs.is_none() {
            return Err("Fragment stage was not specified!");
        }

        let vs_module = device.create_shader_module(&self.vs.unwrap().bytes);
        let vertex_stage = wgpu::ProgrammableStageDescriptor {
            module: &vs_module,
            entry_point: "main"
        };

        let fs_module = device.create_shader_module(&self.fs.unwrap().bytes);
        let fragment_stage = Some(wgpu::ProgrammableStageDescriptor {
            module: &fs_module,
            entry_point: "main"
        });

        let mut attributes = Vec::new();
        let mut cur_offset = 0;

        self.vertex_attributes.sort_by(|a, b| a.0.cmp(&b.0));
        for attribute in &self.vertex_attributes {
            attributes.push(
                wgpu::VertexAttributeDescriptor {
                    offset: cur_offset,
                    format: attribute.1,
                    shader_location: attribute.0
                }
            );
            cur_offset += match attribute.1 {
                VertexFormat::Uchar2 => 2,
                VertexFormat::Uchar4 => 4,
                VertexFormat::Char2 => 2,
                VertexFormat::Char4 => 4,
                VertexFormat::Uchar2Norm => 2,
                VertexFormat::Uchar4Norm => 4,
                VertexFormat::Char2Norm => 2,
                VertexFormat::Char4Norm => 4,
                VertexFormat::Ushort2 => 4,
                VertexFormat::Ushort4 => 8,
                VertexFormat::Short2 => 4,
                VertexFormat::Short4 => 8,
                VertexFormat::Ushort2Norm => 4,
                VertexFormat::Ushort4Norm => 8,
                VertexFormat::Short2Norm => 4,
                VertexFormat::Short4Norm => 8,
                VertexFormat::Half2 => 4,
                VertexFormat::Half4 => 8,
                VertexFormat::Float => 4,
                VertexFormat::Float2 => 8,
                VertexFormat::Float3 => 12,
                VertexFormat::Float4 => 16,
                VertexFormat::Uint => 4,
                VertexFormat::Uint2 => 8,
                VertexFormat::Uint3 => 12,
                VertexFormat::Uint4 => 16,
                VertexFormat::Int => 4,
                VertexFormat::Int2 => 8,
                VertexFormat::Int3 => 12,
                VertexFormat::Int4 => 16,
            }
        }

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage,
            fragment_stage,
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: self.culling.0,
                cull_mode: self.culling.1,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[
                wgpu::ColorStateDescriptor {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    alpha_blend: wgpu::BlendDescriptor::REPLACE,
                    color_blend: wgpu::BlendDescriptor::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL
                }
            ],
            depth_stencil_state: None,
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[
                wgpu::VertexBufferDescriptor {
                    stride: cur_offset as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &attributes
                }
            ],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false
        });

        Ok(Renderer {
            device,
            queue,
            bind_group,
            bind_group_layout,
            pipeline,
            swap_chain,
            uniform_buffer,
            uniform_location: self.uniform_location,
            textures,
            sampler,
            sampler_location: self.sampler_location,
            phantom: PhantomData
        })
    }
}