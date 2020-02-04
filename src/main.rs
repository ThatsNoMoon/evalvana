use std::time::Instant;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use wgpu_glyph::{
    Section,
    GlyphBrushBuilder,
    Scale as FontScale,
    SectionText,
    VariedSection,
};

use cgmath::{Vector2, vec2};

mod config;
use config::{Config, UiColors, EditorColors};

mod color;
use color::Color;

const RENDER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    pos: Vector2<f32>,
    color: Color,
}

fn main() {
    env_logger::init();

    let config = Config {
        ui_colors: UiColors {
            bg: Color::from_rgb_u32(0x282C34),
            text: Color::from_rgb_u32(0xABB2BF),
            borders: Color::from_rgb_u32(0x4B5263),
        },
        editor_colors: EditorColors {
            bg: Color::from_rgb_u32(0x282C34),
            main: Color::from_rgb_u32(0xABB2BF),
            strings: Color::from_rgb_u32(0x98C379),
            numbers: Color::from_rgb_u32(0xD19A66),
            operators: Color::from_rgb_u32(0xC678DD),
            keywords: Color::from_rgb_u32(0xE06C75),
            variables: Color::from_rgb_u32(0xE5C07B),
            parameters: Color::from_rgb_u32(0xE5C07B),
            constants: Color::from_rgb_u32(0x56B6C2),
            types: Color::from_rgb_u32(0x61AFEF),
            functions: Color::from_rgb_u32(0xABB2BF),
        }
    };

    let text = std::env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("Hello, world!"));

    const FONT: &'static [u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts/JetBrainsMono-Regular.ttf"));

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let size = window.inner_size();
    let surface = wgpu::Surface::create(&window);

    let adapter = wgpu::Adapter::request(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            backends: wgpu::BackendBit::PRIMARY,
        },
    ).unwrap();

    let (mut device, mut queue) = adapter.request_device(&wgpu::DeviceDescriptor {
        extensions: wgpu::Extensions {
            anisotropic_filtering: false,
        },
        limits: wgpu::Limits::default(),
    });

    let mut glyph_brush = GlyphBrushBuilder::using_font_bytes(FONT)
        .build(&mut device, RENDER_FORMAT);

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

    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: RENDER_FORMAT,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Vsync,
    };

    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

    let render_start = Instant::now();

    let mut frame_count = 0u64;

    let mut last_frame = Instant::now();

    let mut delta_times = [0f32; 20];

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::MainEventsCleared => window.request_redraw(),

            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                sc_desc.width = size.width;
                sc_desc.height = size.height;
                swap_chain = device.create_swap_chain(&surface, &sc_desc);
            },

            Event::RedrawRequested(_) => {
                let frame = swap_chain
                    .get_next_texture();
                let mut encoder = device.create_command_encoder(
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
                    rpass.set_pipeline(&render_pipeline);
                    rpass.set_bind_group(0, &bind_group, &[]);
                    rpass.set_vertex_buffers(0, &[(&vertex_buffer, 0)]);
                    rpass.set_index_buffer(&index_buffer, 0);
                    rpass.draw_indexed(0..indices.len() as u32, 0, 0..1);
                }

                {
                    let text_pos = clip_to_pixel_coordinates(vec2(-0.5, -0.6), sc_desc.width, sc_desc.height);
                    let text_section = Section {
                        text: text.as_str(),
                        screen_position: (text_pos.x, text_pos.y),
                        color: config.editor_colors.main.to_rgba(),
                        scale: FontScale::uniform(40.0),
                        ..Section::default()
                    };
                    
                    glyph_brush.queue(text_section);

                    let delta = last_frame.elapsed().as_secs_f32();
                    let range = 1..delta_times.len();
                    delta_times.copy_within(range, 0);
                    *delta_times.last_mut().unwrap() = delta;

                    let avg_delta = delta_times.iter().sum::<f32>() / delta_times.len() as f32;

                    let lifetime_fps = frame_count / (render_start.elapsed().as_secs() + 1 as u64);

                    frame_count += 1;

                    let fps = format!(
                        "current fps: {}\nlifetime fps: {}",
                        (1.0 / avg_delta) as u32,
                        lifetime_fps);

                    let fps_counter = Section {
                        text: fps.as_str(),
                        scale: FontScale::uniform(15.0),
                        color: config.ui_colors.text.to_rgba(),
                        ..Section::default()
                    };

                    glyph_brush.queue(fps_counter);

                    glyph_brush.draw_queued(
                        &mut device,
                        &mut encoder,
                        &frame.view,
                        sc_desc.width,
                        sc_desc.height,
                    ).unwrap();
                }

                queue.submit(&[encoder.finish()]);

                last_frame = Instant::now();
            },

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}

fn clip_to_pixel_coordinates(clip: cgmath::Vector2<f32>, width: u32, height: u32) -> cgmath::Vector2<f32> {
    vec2((clip.x + 1.0) / 2.0 * (width as f32), (clip.y + 1.0) / 2.0 * (height as f32))
}
