pub mod color;
use color::Color;

mod pipelines;
use pipelines::Pipelines;

mod bounding_box;

pub mod text;
use text::TextRenderer;

mod drawable;
use drawable::{Drawable, DrawingContext};

use crate::config::Config;
use crate::icons::Icons;
use crate::interface::Interface;

use std::time::Instant;

use crossbeam_channel::bounded as bounded_channel;
use wgpu_glyph::{
	GlyphBrushBuilder, Scale as FontScale, Section, SectionText, VariedSection,
};
use winit::{dpi::PhysicalSize, window::Window};
use zerocopy::{AsBytes, FromBytes};

const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

pub struct Renderer {
	surface: wgpu::Surface,
	adapter: wgpu::Adapter,
	device: wgpu::Device,
	queue: wgpu::Queue,
	pipelines: Pipelines,
	swap_chain_descriptor: wgpu::SwapChainDescriptor,
	swap_chain: wgpu::SwapChain,

	text_renderer: TextRenderer,

	color_vertices: Option<Vec<Vertex>>,
	color_indices: Option<Vec<VertexIndex>>,
	color_vertex_buffer: wgpu::Buffer,
	color_index_buffer: wgpu::Buffer,

	texture_vertices: Option<Vec<TextureVertex>>,
	texture_indices: Option<Vec<VertexIndex>>,
	texture_vertex_buffer: wgpu::Buffer,
	texture_index_buffer: wgpu::Buffer,

	last_frame: Instant,
	delta_times: [f32; 20],
}

impl Renderer {
	pub fn new(window: &Window, icons: &Icons) -> Renderer {
		let size = window.inner_size();
		let surface = wgpu::Surface::create(window);

		let adapter = wgpu::Adapter::request(&wgpu::RequestAdapterOptions {
			power_preference: wgpu::PowerPreference::Default,
			backends: wgpu::BackendBit::PRIMARY,
		})
		.unwrap();

		let (mut device, mut queue) =
			adapter.request_device(&wgpu::DeviceDescriptor {
				extensions: wgpu::Extensions {
					anisotropic_filtering: false,
				},
				limits: wgpu::Limits::default(),
			});

		let text_renderer = TextRenderer::new(&mut device, COLOR_FORMAT);

		let pipelines = Pipelines::new(&device, icons.texture_atlas_size());

		{
			let mut encoder = device.create_command_encoder(
				&wgpu::CommandEncoderDescriptor { todo: 0 },
			);

			icons.fill_texture_atlas(
				&device,
				pipelines.texture_atlas(),
				&mut encoder,
			);

			queue.submit(&[encoder.finish()]);
		}

		device.poll(true);

		let color_vertex_buffer =
			device.create_buffer(&wgpu::BufferDescriptor {
				size: 500
					* std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
				usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::MAP_WRITE,
			});

		let color_index_buffer =
			device.create_buffer(&wgpu::BufferDescriptor {
				size: 500
					* std::mem::size_of::<VertexIndex>() as wgpu::BufferAddress,
				usage: wgpu::BufferUsage::INDEX | wgpu::BufferUsage::MAP_WRITE,
			});

		let texture_vertex_buffer =
			device.create_buffer(&wgpu::BufferDescriptor {
				size: 500
					* std::mem::size_of::<TextureVertex>()
						as wgpu::BufferAddress,
				usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::MAP_WRITE,
			});

		let texture_index_buffer =
			device.create_buffer(&wgpu::BufferDescriptor {
				size: 500
					* std::mem::size_of::<VertexIndex>() as wgpu::BufferAddress,
				usage: wgpu::BufferUsage::INDEX | wgpu::BufferUsage::MAP_WRITE,
			});

		let sc_desc = wgpu::SwapChainDescriptor {
			usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
			format: COLOR_FORMAT,
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
			pipelines,
			swap_chain_descriptor: sc_desc,
			swap_chain,

			text_renderer,

			color_vertices: Some(vec![]),
			color_indices: Some(vec![]),
			color_vertex_buffer,
			color_index_buffer,

			texture_vertices: Some(vec![]),
			texture_indices: Some(vec![]),
			texture_vertex_buffer,
			texture_index_buffer,

			last_frame,
			delta_times,
		}
	}

	pub fn redraw(
		&mut self,
		window: &Window,
		config: &Config,
		icons: &Icons,
		interface: &mut Interface,
	) {
		log::trace!("redrawing");

		let mut color_vertices = self
			.color_vertices
			.take()
			.expect("Color vertex list left empty by last draw cycle");
		let mut color_indices = self
			.color_indices
			.take()
			.expect("Color index list left empty by last draw cycle");
		let mut texture_vertices = self
			.texture_vertices
			.take()
			.expect("Texture vertex list left empty by last draw cycle");
		let mut texture_indices = self
			.texture_indices
			.take()
			.expect("Texture index list left empty by last draw cycle");
		color_vertices.clear();
		color_indices.clear();
		texture_vertices.clear();
		texture_indices.clear();

		let ctx = DrawingContext::new(
			window,
			config,
			icons,
			&mut color_vertices,
			&mut color_indices,
			&mut texture_vertices,
			&mut texture_indices,
			&mut self.text_renderer,
		);
		interface.draw(ctx, ());

		let n_color_indices = color_indices.len() as u32;
		let n_texture_indices = texture_indices.len() as u32;

		enum ListMessage {
			ColorVertexList(Vec<Vertex>),
			ColorIndexList(Vec<VertexIndex>),
			TextureVertexList(Vec<TextureVertex>),
			TextureIndexList(Vec<VertexIndex>),
		}

		let (list_sender, list_receiver) = bounded_channel(4);

		let list_sender_clone = list_sender.clone();
		self.color_vertex_buffer.map_write_async(
			0,
			color_vertices.as_bytes().len() as wgpu::BufferAddress,
			move |vertex_buffer| {
				vertex_buffer
					.unwrap()
					.data
					.copy_from_slice(color_vertices.as_slice());
				list_sender_clone
					.send(ListMessage::ColorVertexList(color_vertices))
					.expect("Failed to send color vertex list");
			},
		);

		let list_sender_clone = list_sender.clone();
		self.color_index_buffer.map_write_async(
			0,
			color_indices.as_bytes().len() as wgpu::BufferAddress,
			move |color_index_buffer| {
				color_index_buffer
					.unwrap()
					.data
					.copy_from_slice(color_indices.as_slice());
				list_sender_clone
					.send(ListMessage::ColorIndexList(color_indices))
					.expect("Failed to send color index list");
			},
		);

		let list_sender_clone = list_sender.clone();
		self.texture_vertex_buffer.map_write_async(
			0,
			texture_vertices.as_bytes().len() as wgpu::BufferAddress,
			move |vertex_buffer| {
				vertex_buffer
					.unwrap()
					.data
					.copy_from_slice(texture_vertices.as_slice());
				list_sender_clone
					.send(ListMessage::TextureVertexList(texture_vertices))
					.expect("Failed to send texture vertex list");
			},
		);

		let list_sender_clone = list_sender.clone();
		self.texture_index_buffer.map_write_async(
			0,
			texture_indices.as_bytes().len() as wgpu::BufferAddress,
			move |texture_index_buffer| {
				texture_index_buffer
					.unwrap()
					.data
					.copy_from_slice(texture_indices.as_slice());
				list_sender_clone
					.send(ListMessage::TextureIndexList(texture_indices))
					.expect("Failed to send texture index list");
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
							clear_color: wgpu::Color::BLACK,
						},
					],
					depth_stencil_attachment: None,
				});
			rpass.set_pipeline(&self.pipelines.color_pipeline());
			rpass.set_bind_group(0, &self.pipelines.color_bind_group(), &[]);
			rpass.set_vertex_buffers(0, &[(&self.color_vertex_buffer, 0)]);
			rpass.set_index_buffer(&self.color_index_buffer, 0);
			rpass.draw_indexed(0..n_color_indices, 0, 0..1);
		}

		{
			let mut rpass =
				encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
					color_attachments: &[
						wgpu::RenderPassColorAttachmentDescriptor {
							attachment: &frame.view,
							resolve_target: None,
							load_op: wgpu::LoadOp::Load,
							store_op: wgpu::StoreOp::Store,
							clear_color: wgpu::Color {
								r: 0.0,
								g: 0.0,
								b: 0.0,
								a: 0.0,
							},
						},
					],
					depth_stencil_attachment: None,
				});
			rpass.set_pipeline(self.pipelines.texture_pipeline());
			rpass.set_bind_group(0, self.pipelines.texture_bind_group(), &[]);
			rpass.set_vertex_buffers(0, &[(&self.texture_vertex_buffer, 0)]);
			rpass.set_index_buffer(&self.texture_index_buffer, 0);
			rpass.draw_indexed(0..n_texture_indices, 0, 0..1);
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
		}

		self.device.poll(true);

		for _ in 0..4 {
			match list_receiver
				.recv()
				.expect("Failed to receive vertex/index list")
			{
				ListMessage::ColorVertexList(list) => {
					self.color_vertices = Some(list)
				}
				ListMessage::ColorIndexList(list) => {
					self.color_indices = Some(list)
				}
				ListMessage::TextureVertexList(list) => {
					self.texture_vertices = Some(list)
				}
				ListMessage::TextureIndexList(list) => {
					self.texture_indices = Some(list)
				}
			}
		}

		self.color_vertex_buffer.unmap();
		self.color_index_buffer.unmap();
		self.texture_vertex_buffer.unmap();
		self.texture_index_buffer.unmap();

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

impl Point {
	pub fn new(x: f32, y: f32) -> Point {
		Point { x, y }
	}
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

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, AsBytes, FromBytes)]
pub struct TextureVertex {
	pos: Point,
	tex_coord: Point,
}

impl TextureVertex {
	pub fn new(pos: Point, tex_coord: Point) -> TextureVertex {
		TextureVertex { pos, tex_coord }
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
