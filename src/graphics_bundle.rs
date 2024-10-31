use glam::Vec2;
use image::{DynamicImage, GenericImageView};
use wgpu::{
    util::DeviceExt, BindGroupDescriptor, BindGroupLayoutDescriptor, Device, PipelineLayout,
    PipelineLayoutDescriptor, PrimitiveTopology, RenderPipeline, TextureFormat,
};

use crate::{texture, Drag, Selection};

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Default, Debug)]
pub struct SelectionUniforms {
    screen_size: Vec2,
    drag_start: Vec2,
    drag_end: Vec2,
    selection_start: Vec2,
    selection_end: Vec2,
    time: f32,
    is_dragging: u32, // 0 = None, 1 = Dragging, 2 = Selected, 3 = Both
}

impl std::fmt::Display for SelectionUniforms {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "size: {:?}, is_dragging: {}, drag_start: {:?}, drag_end: {:?}, selection_start: {:?}, selection_end: {:?}, time: {}", 
          self.screen_size, self.is_dragging, self.drag_start, self.drag_end, self.selection_start, self.selection_end, self.time)
    }
}

pub struct GraphicsBundle {
    pipeline: wgpu::RenderPipeline,
    texture_bind_group: wgpu::BindGroup,
    uniform_bind_group: wgpu::BindGroup,
    texture: texture::Texture,
    uniforms: SelectionUniforms,
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
        let (w, h) = img.dimensions();
        let screen_size = Vec2::new(w as f32, h as f32);
        let uniforms = SelectionUniforms {
            screen_size,
            is_dragging: 0,
            drag_start: Vec2::ZERO,
            drag_end: Vec2::ZERO,
            selection_start: Vec2::ZERO,
            selection_end: Vec2::ZERO,
            time: 0.0,
        };
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            layout: &uniform_bind_group_layout,
            label: None,
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
            layout: &bind_group_layout,
            label: None,
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
            bind_group_layouts: &[&bind_group_layout, &uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = make_pipeline(device, format, &layout, topology);

        Self {
            pipeline,
            texture_bind_group: bind_group,
            uniform_bind_group,
            texture,
            uniforms,
            uniform_buffer,
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn update_selection(
        &mut self,
        time: f32,
        queue: &wgpu::Queue,
        drag: Option<&Drag>,
        selection: Option<&Selection>,
        size: Vec2,
    ) {
        self.uniforms.time = time;
        self.uniforms.screen_size = size;
        self.uniforms.is_dragging = match (drag, selection) {
            (Some(_), Some(_)) => 3,
            (Some(_), None) => 1,
            (None, Some(_)) => 2,
            (None, None) => 0,
        };

        // if let Some(drag) = drag {
        //     self.uniforms.selection_start = Vec2::new(drag.start.0 as f32, drag.start.1 as f32);
        //     if let Some(end) = drag.end {
        //         self.uniforms.selection_end = Vec2::new(end.0 as f32, end.1 as f32);
        //     }
        // } else if let Some(selection) = selection {
        //     self.uniforms.selection_start =
        //         Vec2::new(selection.start.0 as f32, selection.start.1 as f32);
        //     self.uniforms.selection_end = Vec2::new(selection.end.0 as f32, selection.end.1 as f32);
        // }
        if let Some(drag) = drag {
            self.uniforms.drag_start = Vec2::new(drag.start.0 as f32, drag.start.1 as f32);
            if let Some(end) = drag.end {
                self.uniforms.drag_end = Vec2::new(end.0 as f32, end.1 as f32);
            }
        }
        if let Some(selection) = selection {
            let ((min_x, min_y), (max_x, max_y)) = selection.coords();
            self.uniforms.selection_start = Vec2::new(min_x as f32, min_y as f32);
            self.uniforms.selection_end = Vec2::new(max_x as f32, max_y as f32);
        }

        // println!("{}", self.uniforms);

        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );
    }

    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.texture_bind_group, &[]);
        pass.set_bind_group(1, &self.uniform_bind_group, &[]);
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
        entry_point: Some("vs_main"),
        buffers: &[
            crate::vertex::Vertex::desc(),
            // crate::instance::InstanceRaw::desc(),
        ],
        compilation_options: wgpu::PipelineCompilationOptions::default(),
    };
    let fragment = wgpu::FragmentState {
        module: &shader,
        entry_point: Some("fs_main"),
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
