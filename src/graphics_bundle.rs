use glam::Vec2;
use image::{DynamicImage, GenericImageView};
use wgpu::{
    util::DeviceExt, BindGroupDescriptor, BindGroupLayoutDescriptor, Device, PipelineLayout,
    PipelineLayoutDescriptor, PrimitiveTopology, RenderPipeline, TextureFormat,
};

use crate::texture;

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
struct Uniforms {
    pub resolution: Vec2,
    pub time: f32,
}

fn load_texture(
    img: image::DynamicImage,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> wgpu::Texture {
    let rgba = img.to_rgba8();
    let dimensions = img.dimensions();
    let size = wgpu::Extent3d {
        width: dimensions.0,
        height: dimensions.1,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        label: None,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &rgba,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * dimensions.0),
            rows_per_image: Some(dimensions.1),
        },
        size,
    );
    texture
}

pub struct GraphicsBundle {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    texture: texture::Texture,
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl GraphicsBundle {
    pub fn new(
        img: DynamicImage,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        topology: PrimitiveTopology,
        format: TextureFormat,
    ) -> Self {
        // let texture = load_texture(img, device, queue);
        // let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        // let texture_sampler = device.create_sampler(&SamplerDescriptor {
        //     address_mode_u: wgpu::AddressMode::ClampToEdge,
        //     address_mode_v: wgpu::AddressMode::ClampToEdge,
        //     address_mode_w: wgpu::AddressMode::ClampToEdge,
        //     mag_filter: wgpu::FilterMode::Linear,
        //     min_filter: wgpu::FilterMode::Nearest,
        //     mipmap_filter: wgpu::FilterMode::Nearest,
        //     ..Default::default()
        // });
        let texture = texture::Texture::from_image(device, queue, &img, None)
            .expect("Could not load texture");
        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            layout: &texture_bind_group_layout,
            label: None,
        });
        let uniforms = Uniforms {
            resolution: Vec2::new(800., 600.),
            time: 0.,
        };
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&crate::vertex::QUAD),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&crate::vertex::QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = make_pipeline(device, format, &layout, topology);

        Self {
            pipeline,
            bind_group,
            texture,
            uniforms,
            uniform_buffer,
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn update_buffers(&mut self, time: f32, queue: &wgpu::Queue) {
        self.uniforms.time = time;
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );
    }

    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        // pass.set_vertex_buffer(1, self.uniform_buffer.slice(..));
        // pass.draw(0..4, 0..1);
        pass.draw_indexed(0..4, 0, 0..1);
    }
}

fn make_pipeline(
    device: &Device,
    format: TextureFormat,
    // config: &SurfaceConfiguration,
    layout: &PipelineLayout,
    topology: PrimitiveTopology,
) -> RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/gui.wgsl").into()),
    });
    let vertex = wgpu::VertexState {
        module: &shader,
        entry_point: "vs_main",
        buffers: &[
            crate::vertex::Vertex::desc(),
            // crate::instance::InstanceRaw::desc(),
        ],
        compilation_options: wgpu::PipelineCompilationOptions::default(),
    };
    let fragment = wgpu::FragmentState {
        module: &shader,
        entry_point: "fs_main",
        targets: &[Some(wgpu::ColorTargetState {
            format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })],
        compilation_options: wgpu::PipelineCompilationOptions::default(),
    };

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(layout),
        vertex,
        fragment: Some(fragment),
        primitive: wgpu::PrimitiveState {
            topology,
            strip_index_format: topology.is_strip().then_some(wgpu::IndexFormat::Uint16),
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
            unclipped_depth: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    })
}
