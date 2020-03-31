pub mod color;
use color::Color;

pub mod text;
use text::TextRenderer;

use crate::config::Config;

use std::time::Instant;

use cgmath::{Vector2, vec2};
use winit::{
    window::Window,
    dpi::PhysicalSize,
};
use wgpu_glyph::{
    Section,
    GlyphBrushBuilder,
    Scale as FontScale,
    SectionText,
    VariedSection,
};


const RENDER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

const FONT: &'static [u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts/JetBrainsMono-Regular.ttf"));

pub struct Renderer {
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    vertex_shader_module: wgpu::ShaderModule,
    fragment_shader_module: wgpu::ShaderModule,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    swap_chain_descriptor: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,

    text_renderer: TextRenderer,

    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    last_frame: Instant,
    delta_times: [f32; 20],
}

impl Renderer {
    pub fn new(window: &Window, size: PhysicalSize<u32>) -> Renderer {
        let surface = wgpu::Surface::create(window);

        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                backends: wgpu::BackendBit::PRIMARY,
            },
        ).unwrap();

        let (mut device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
            limits: wgpu::Limits::default(),
        });

        let text_renderer = TextRenderer::new(&mut device, FONT, RENDER_FORMAT);

        let vs = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/target/shaders/shader.vert.spv"));
        let vs_module = device.create_shader_module(
            &wgpu::read_spirv(std::io::Cursor::new(&vs[..])).unwrap()
        );

        let fs = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/target/shaders/shader.frag.spv"));
        let fs_module = device.create_shader_module(
            &wgpu::read_spirv(std::io::Cursor::new(&fs[..])).unwrap()
        );

        let bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor { bindings: &[] }
        );
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[],
        });
        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&bind_group_layout],
            }
        );

        let render_pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                layout: &pipeline_layout,
                vertex_stage: wgpu::ProgrammableStageDescriptor {
                    module: &vs_module,
                    entry_point: "main",
                },
                fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                    module: &fs_module,
                    entry_point: "main",
                }),
                rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: wgpu::CullMode::None,
                    depth_bias: 0,
                    depth_bias_slope_scale: 0.0,
                    depth_bias_clamp: 0.0,
                }),
                primitive_topology: wgpu::PrimitiveTopology::TriangleList,
                color_states: &[wgpu::ColorStateDescriptor {
                    format: RENDER_FORMAT,
                    color_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::Zero,
                        operation: wgpu::BlendOperation::Add,
                    },
                    write_mask: wgpu::ColorWrite::ALL,
                }],
                depth_stencil_state: None,
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[
                    wgpu::VertexBufferDescriptor {
                        stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::InputStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttributeDescriptor {
                                offset: memoffset::offset_of!(Vertex, pos) as wgpu::BufferAddress,
                                format: wgpu::VertexFormat::Float2,
                                shader_location: 0,
                            },
                            wgpu::VertexAttributeDescriptor {
                                offset: memoffset::offset_of!(Vertex, color) as wgpu::BufferAddress,
                                format: wgpu::VertexFormat::Float3,
                                shader_location: 1,
                            }
                        ]
                    }
                ],
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
            }
        );

        let vertices = [
            Vertex { pos: vec2(-0.5, -0.5), color: Color::from_rgb_u8(0xFF, 0, 0) },
            Vertex { pos: vec2(0.5, -0.5), color: Color::from_rgb_u8(0, 0xFF, 0) },
            Vertex { pos: vec2(0.5, 0.5), color: Color::from_rgb_u8(0, 0, 0xFF) },
            Vertex { pos: vec2(-0.5, 0.5), color: Color::from_rgb_u8(0xFF, 0xFF, 0xFF) },
        ];

        let indices = [
            0u16, 1, 2, 2, 3, 0
        ];

        let vertex_buffer = device.create_buffer_mapped(
            vertices.len(),
            wgpu::BufferUsage::VERTEX,
        )
            .fill_from_slice(&vertices[..]);

        let index_buffer = device.create_buffer_mapped(
            indices.len(),
            wgpu::BufferUsage::INDEX,
        )
            .fill_from_slice(&indices[..]);

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: RENDER_FORMAT,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Vsync,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let last_frame = Instant::now();

        let delta_times = [0f32; 20];

        Renderer {
            surface,
            adapter,
            device,
            queue,
            vertex_shader_module: vs_module,
            fragment_shader_module: fs_module,
            bind_group,
            render_pipeline,
            swap_chain_descriptor: sc_desc,
            swap_chain,

            text_renderer,

            vertices: vertices.to_vec(),
            indices: indices.to_vec(),
            vertex_buffer,
            index_buffer,

            last_frame,
            delta_times,
        }
    }

   pub fn redraw(&mut self, config: &Config) {
        let frame = self.swap_chain
            .get_next_texture();
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { todo: 0 }
        );
        {
            let mut rpass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.view,
                        resolve_target: None,
                        load_op: wgpu::LoadOp::Clear,
                        store_op: wgpu::StoreOp::Store,
                        clear_color: config.ui_colors.bg.to_wgpu(),
                    }],
                    depth_stencil_attachment: None,
                }
            );
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_vertex_buffers(0, &[(&self.vertex_buffer, 0)]);
            rpass.set_index_buffer(&self.index_buffer, 0);
            rpass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);
        }

        {
            let text_pos = clip_to_pixel_coordinates(vec2(-0.5, -0.6), self.swap_chain_descriptor.width, self.swap_chain_descriptor.height);
            let text_section = Section {
                text: "Hello, world!",
                screen_position: (text_pos.x, text_pos.y),
                color: config.editor_colors.main.to_rgba(),
                scale: FontScale::uniform(40.0),
                ..Section::default()
            };
            
            self.text_renderer.queue(text_section);

            let delta = self.last_frame.elapsed().as_secs_f32();
            let range = 1..self.delta_times.len();
            self.delta_times.copy_within(range, 0);
            *self.delta_times.last_mut().unwrap() = delta;

            let avg_delta = self.delta_times.iter().sum::<f32>() / self.delta_times.len() as f32;

            let fps = format!(
                "current fps: {}",
                (1.0 / avg_delta) as u32);

            let fps_counter = Section {
                text: fps.as_str(),
                scale: FontScale::uniform(15.0),
                color: config.ui_colors.text.to_rgba(),
                ..Section::default()
            };

            self.text_renderer.queue(fps_counter);

            self.text_renderer.draw_queued(
                &mut self.device,
                &mut encoder,
                &frame.view,
                self.swap_chain_descriptor.width,
                self.swap_chain_descriptor.height,
            ).unwrap();
        }

        self.queue.submit(&[encoder.finish()]);

        self.last_frame = Instant::now();
    } 

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.swap_chain_descriptor.width = size.width;
        self.swap_chain_descriptor.height = size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.swap_chain_descriptor);
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pos: Vector2<f32>,
    color: Color,
}

fn clip_to_pixel_coordinates(clip: cgmath::Vector2<f32>, width: u32, height: u32) -> cgmath::Vector2<f32> {
    vec2((clip.x + 1.0) / 2.0 * (width as f32), (clip.y + 1.0) / 2.0 * (height as f32))
}