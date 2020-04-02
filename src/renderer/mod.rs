pub mod color;
use color::Color;

pub mod text;
use text::TextRenderer;

mod drawable;
use drawable::{Drawable, DrawingContext};

use crate::config::Config;
use crate::interface::Interface;

use std::time::Instant;

use crossbeam_channel::bounded as bounded_channel;
use wgpu_glyph::{
	GlyphBrushBuilder, Scale as FontScale, Section, SectionText, VariedSection,
};
use winit::{dpi::PhysicalSize, window::Window};
use zerocopy::{AsBytes, FromBytes};

const RENDER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

const FONT: &'static [u8] = include_bytes!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/assets/fonts/JetBrainsMono-Regular.ttf"
));

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

	vertices: Option<Vec<Vertex>>,
	indices: Option<Vec<VertexIndex>>,
	vertex_buffer: wgpu::Buffer,
	index_buffer: wgpu::Buffer,

	last_frame: Instant,
	delta_times: [f32; 20],
}

impl Renderer {
	pub fn new(window: &Window) -> Renderer {
		let size = window.inner_size();
		let surface = wgpu::Surface::create(window);

		let adapter = wgpu::Adapter::request(&wgpu::RequestAdapterOptions {
			power_preference: wgpu::PowerPreference::Default,
			backends: wgpu::BackendBit::PRIMARY,
		})
		.unwrap();

		let (mut device, queue) =
			adapter.request_device(&wgpu::DeviceDescriptor {
				extensions: wgpu::Extensions {
					anisotropic_filtering: false,
				},
				limits: wgpu::Limits::default(),
			});

		let text_renderer = TextRenderer::new(&mut device, FONT, RENDER_FORMAT);

		let vs = include_bytes!(concat!(
			env!("CARGO_MANIFEST_DIR"),
			"/target/shaders/shader.vert.spv"
		));
		let vs_module = device.create_shader_module(
			&wgpu::read_spirv(std::io::Cursor::new(&vs[..])).unwrap(),
		);

		let fs = include_bytes!(concat!(
			env!("CARGO_MANIFEST_DIR"),
			"/target/shaders/shader.frag.spv"
		));
		let fs_module = device.create_shader_module(
			&wgpu::read_spirv(std::io::Cursor::new(&fs[..])).unwrap(),
		);

		let bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				bindings: &[],
			});
		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &bind_group_layout,
			bindings: &[],
		});
		let pipeline_layout =
			device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				bind_group_layouts: &[&bind_group_layout],
			});

		let render_pipeline =
			device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
				index_format: VertexIndex::WGPU_FORMAT,
				vertex_buffers: &[wgpu::VertexBufferDescriptor {
					stride: std::mem::size_of::<Vertex>()
						as wgpu::BufferAddress,
					step_mode: wgpu::InputStepMode::Vertex,
					attributes: &[
						wgpu::VertexAttributeDescriptor {
							offset: memoffset::offset_of!(Vertex, pos)
								as wgpu::BufferAddress,
							format: wgpu::VertexFormat::Float2,
							shader_location: 0,
						},
						wgpu::VertexAttributeDescriptor {
							offset: memoffset::offset_of!(Vertex, color)
								as wgpu::BufferAddress,
							format: wgpu::VertexFormat::Float3,
							shader_location: 1,
						},
					],
				}],
				sample_count: 1,
				sample_mask: !0,
				alpha_to_coverage_enabled: false,
			});

		let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			size: 500 * std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
			usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::MAP_WRITE,
		});

		let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			size: 500
				* std::mem::size_of::<VertexIndex>() as wgpu::BufferAddress,
			usage: wgpu::BufferUsage::INDEX | wgpu::BufferUsage::MAP_WRITE,
		});

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

			vertices: Some(vec![]),
			indices: Some(vec![]),
			vertex_buffer,
			index_buffer,

			last_frame,
			delta_times,
		}
	}

	pub fn redraw(
		&mut self,
		window: &Window,
		config: &Config,
		interface: &mut Interface,
	) {
		let mut vertices = self.vertices.take().unwrap();
		let mut indices = self.indices.take().unwrap();
		vertices.clear();
		indices.clear();
		let ctx = DrawingContext::new(
			window,
			config,
			&mut vertices,
			&mut indices,
			&mut self.text_renderer,
		);
		interface.draw(ctx);
		let n_indices = indices.len() as u32;

		let (vertex_sender, vertex_receiver) = bounded_channel(1);
		let (index_sender, index_receiver) = bounded_channel(1);

		self.vertex_buffer.map_write_async(
			0,
			vertices.as_bytes().len() as wgpu::BufferAddress,
			move |vertex_buffer| {
				vertex_buffer
					.unwrap()
					.data
					.copy_from_slice(vertices.as_slice());
				vertex_sender.send(vertices);
			},
		);

		self.index_buffer.map_write_async(
			0,
			indices.as_bytes().len() as wgpu::BufferAddress,
			move |index_buffer| {
				index_buffer
					.unwrap()
					.data
					.copy_from_slice(indices.as_slice());
				index_sender.send(indices);
			},
		);

		let frame = self.swap_chain.get_next_texture();
		let mut encoder = self.device.create_command_encoder(
			&wgpu::CommandEncoderDescriptor { todo: 0 },
		);
		{
			let mut rpass =
				encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
					color_attachments: &[
						wgpu::RenderPassColorAttachmentDescriptor {
							attachment: &frame.view,
							resolve_target: None,
							load_op: wgpu::LoadOp::Clear,
							store_op: wgpu::StoreOp::Store,
							clear_color: config.ui_colors.bg.to_wgpu(),
						},
					],
					depth_stencil_attachment: None,
				});
			rpass.set_pipeline(&self.render_pipeline);
			rpass.set_bind_group(0, &self.bind_group, &[]);
			rpass.set_vertex_buffers(0, &[(&self.vertex_buffer, 0)]);
			rpass.set_index_buffer(&self.index_buffer, 0);
			rpass.draw_indexed(0..n_indices, 0, 0..1);
		}

		{
			self.text_renderer
				.draw_queued(
					&mut self.device,
					&mut encoder,
					&frame.view,
					self.swap_chain_descriptor.width,
					self.swap_chain_descriptor.height,
				)
				.expect("Failed to draw glyphs");
			/*
			let text_pos = clip_to_pixel_coordinates(vec2(-0.5, -0.75), self.swap_chain_descriptor.width, self.swap_chain_descriptor.height);
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
			*/
		}

		self.device.poll(true);

		self.vertices = Some(
			vertex_receiver
				.recv()
				.expect("Failed to receive vertex list"),
		);
		self.indices =
			Some(index_receiver.recv().expect("Failed to receive index list"));

		self.vertex_buffer.unmap();
		self.index_buffer.unmap();

		self.queue.submit(&[encoder.finish()]);

		let delta = self.last_frame.elapsed().as_secs_f32();
		let range = 1..self.delta_times.len();
		self.delta_times.copy_within(range, 0);
		*self.delta_times.last_mut().unwrap() = delta;

		self.last_frame = Instant::now();
	}

	pub fn resize(&mut self, size: PhysicalSize<u32>) {
		self.swap_chain_descriptor.width = size.width;
		self.swap_chain_descriptor.height = size.height;
		self.swap_chain = self
			.device
			.create_swap_chain(&self.surface, &self.swap_chain_descriptor);
	}
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, AsBytes, FromBytes)]
pub struct Point {
	x: f32,
	y: f32,
}

pub fn point(x: f32, y: f32) -> Point {
	Point { x, y }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, AsBytes, FromBytes)]
pub struct Vertex {
	pos: Point,
	color: Color,
}

impl Vertex {
	pub fn new(pos: Point, color: Color) -> Vertex {
		Vertex { pos, color }
	}
}

pub type VertexIndex = u16;

trait VertexIndexFormat {
	const WGPU_FORMAT: wgpu::IndexFormat;
}

impl VertexIndexFormat for u16 {
	const WGPU_FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint16;
}

impl VertexIndexFormat for u32 {
	const WGPU_FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint32;
}
