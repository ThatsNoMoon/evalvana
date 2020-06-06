pub mod color;
use color::Color;

mod pipelines;
use pipelines::Pipelines;

mod text;
use text::TextRenderer;

pub mod drawing;
use drawing::{Drawable, DrawingBuffers, DrawingContext, DrawingManager};

use crate::{
	config::Config,
	geometry::{
		ScreenNormPoint, ScreenPixelPoint, ScreenPixelRect, ScreenPixelSize,
		TexNormPoint,
	},
	icons::Icons,
	interface::Interface,
};

use std::{convert::TryInto, time::Instant};

use crossbeam_channel::bounded as bounded_channel;

use winit::{dpi::PhysicalSize, window::Window};
use zerocopy::{AsBytes, FromBytes};

const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

#[derive(Debug)]
struct ActiveBuffers<T: BufferElement> {
	inner: Vec<(Vec<T>, T::Offset)>,
}

impl<T: BufferElement> ActiveBuffers<T> {
	fn len(&self) -> usize {
		self.inner.len()
	}

	fn total_len(&self) -> usize {
		self.inner.iter().map(|(buffer, _)| buffer.len()).sum()
	}

	fn clear(&mut self) {
		self.inner.clear();
	}

	fn push(&mut self, v: Vec<T>, offset: T::Offset) {
		self.inner.push((v, offset));
	}

	fn iter(&self) -> impl Iterator<Item = &'_ Vec<T>> + '_ {
		self.inner.iter().map(|(buffer, _)| buffer)
	}

	fn bounds_iter(
		&self,
	) -> impl Iterator<Item = (usize, &'_ Vec<T>, usize, T::Offset)> + '_ {
		self.inner
			.iter()
			.scan((0, 0), |(start, end), (buffer, offset)| {
				*start = *end;
				*end += buffer.len();
				Some((*start, buffer, *end, *offset))
			})
	}

	fn drain(&mut self) -> impl Iterator<Item = Vec<T>> + '_ {
		self.inner.drain(..).map(|(buffer, _)| buffer)
	}

	fn as_slice(&self) -> &[(Vec<T>, T::Offset)] {
		self.inner.as_slice()
	}

	fn is_completely_empty(&self) -> bool {
		self.total_len() == 0
	}
}

impl<T: AsBytes + BufferElement> ActiveBuffers<T> {
	fn byte_len(&self) -> usize {
		self.inner
			.iter()
			.map(|(buffer, _)| buffer.as_bytes().len())
			.sum()
	}
}

impl<T: BufferElement> Default for ActiveBuffers<T> {
	fn default() -> ActiveBuffers<T> {
		ActiveBuffers { inner: vec![] }
	}
}

pub struct Renderer {
	surface: wgpu::Surface,
	adapter: wgpu::Adapter,
	device: wgpu::Device,
	queue: wgpu::Queue,
	pipelines: Pipelines,
	swap_chain_descriptor: wgpu::SwapChainDescriptor,
	swap_chain: wgpu::SwapChain,

	text_renderer: TextRenderer,

	pub drawing_manager: DrawingManager,

	color_vertices: Option<ActiveBuffers<ColorVertex>>,
	color_indices: Option<ActiveBuffers<VertexIndex>>,
	color_vertex_buffer: wgpu::Buffer,
	color_index_buffer: wgpu::Buffer,

	texture_vertices: Option<ActiveBuffers<TextureVertex>>,
	texture_indices: Option<ActiveBuffers<VertexIndex>>,
	texture_vertex_buffer: wgpu::Buffer,
	texture_index_buffer: wgpu::Buffer,

	last_frame: Instant,
	delta_times: [f32; 20],
}

impl Renderer {
	pub fn new(window: &Window, icons: &Icons) -> Renderer {
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
					* std::mem::size_of::<ColorVertex>() as wgpu::BufferAddress,
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

		let size = window.inner_size();
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

		let drawing_manager = DrawingManager::default();

		Renderer {
			surface,
			adapter,
			device,
			queue,
			pipelines,
			swap_chain_descriptor: sc_desc,
			swap_chain,

			text_renderer,

			drawing_manager,

			color_vertices: Some(ActiveBuffers::default()),
			color_indices: Some(ActiveBuffers::default()),
			color_vertex_buffer,
			color_index_buffer,

			texture_vertices: Some(ActiveBuffers::default()),
			texture_indices: Some(ActiveBuffers::default()),
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

		{
			let mut ctx = DrawingContext::new(
				window,
				config,
				icons,
				&mut self.drawing_manager,
				&self.text_renderer,
			);

			let window_size =
				window.inner_size().to_logical(window.scale_factor());

			interface.draw(
				&mut ctx,
				ScreenPixelRect::new(
					ScreenPixelPoint::origin(),
					ScreenPixelSize::new(window_size.width, window_size.height),
				),
				(),
			);
		}

		{
			let mut color_vertex_offset = 0;
			let mut texture_vertex_offset = 0;

			for buffers in self.drawing_manager.take_all_drawing_buffers() {
				let DrawingBuffers {
					color_vertex_buffer,
					color_index_buffer,
					texture_vertex_buffer,
					texture_index_buffer,
				} = buffers;
				color_indices.push(color_index_buffer, color_vertex_offset);
				color_vertex_offset += color_vertex_buffer.len() as VertexIndex;
				color_vertices.push(color_vertex_buffer, ());
				texture_indices
					.push(texture_index_buffer, texture_vertex_offset);
				texture_vertex_offset +=
					texture_vertex_buffer.len() as VertexIndex;
				texture_vertices.push(texture_vertex_buffer, ());
			}
		}

		let n_color_indices = color_indices.total_len() as u32;
		let n_texture_indices = texture_indices.total_len() as u32;

		enum ListMessage {
			ColorVertices(ActiveBuffers<ColorVertex>),
			ColorIndices(ActiveBuffers<VertexIndex>),
			TextureVertices(ActiveBuffers<TextureVertex>),
			TextureIndices(ActiveBuffers<VertexIndex>),
		}

		let n_updated_buffers = [
			color_vertices.total_len(),
			color_indices.total_len(),
			texture_vertices.total_len(),
			texture_indices.total_len(),
		]
		.iter()
		.filter(|&&len| len > 0)
		.count();

		let (list_sender, list_receiver) = bounded_channel(n_updated_buffers);

		if !color_vertices.is_completely_empty() {
			let list_sender_clone = list_sender.clone();
			self.color_vertex_buffer.map_write_async(
				0,
				color_vertices.byte_len() as wgpu::BufferAddress,
				move |color_vertex_buffer| {
					let dest = &mut color_vertex_buffer.unwrap().data;
					for (start, src, end, ()) in color_vertices.bounds_iter() {
						dest[start..end].copy_from_slice(src);
					}
					list_sender_clone
						.send(ListMessage::ColorVertices(color_vertices))
						.expect("Failed to send color vertex list");
				},
			);
		} else {
			self.color_vertices = Some(color_vertices);
		}

		if !color_indices.is_completely_empty() {
			let list_sender_clone = list_sender.clone();
			self.color_index_buffer.map_write_async(
				0,
				color_indices.byte_len() as wgpu::BufferAddress,
				move |color_index_buffer| {
					let dest = &mut color_index_buffer.unwrap().data;
					for (start, src, end, offset) in color_indices.bounds_iter()
					{
						dest[start..end].copy_from_slice(src);
						for index in dest[start..end].iter_mut() {
							*index += offset;
						}
					}
					list_sender_clone
						.send(ListMessage::ColorIndices(color_indices))
						.expect("Failed to send color index list");
				},
			);
		} else {
			self.color_indices = Some(color_indices);
		}

		if !texture_vertices.is_completely_empty() {
			let list_sender_clone = list_sender.clone();
			self.texture_vertex_buffer.map_write_async(
				0,
				texture_vertices.byte_len() as wgpu::BufferAddress,
				move |texture_vertex_buffer| {
					let dest = &mut texture_vertex_buffer.unwrap().data;
					for (start, src, end, ()) in texture_vertices.bounds_iter()
					{
						dest[start..end].copy_from_slice(src);
					}
					list_sender_clone
						.send(ListMessage::TextureVertices(texture_vertices))
						.expect("Failed to send texture vertex list");
				},
			);
		} else {
			self.texture_vertices = Some(texture_vertices);
		}

		if !texture_indices.is_completely_empty() {
			self.texture_index_buffer.map_write_async(
				0,
				texture_indices.byte_len() as wgpu::BufferAddress,
				move |texture_index_buffer| {
					let dest = &mut texture_index_buffer.unwrap().data;
					for (start, src, end, offset) in
						texture_indices.bounds_iter()
					{
						dest[start..end].copy_from_slice(src);
						for index in dest[start..end].iter_mut() {
							*index += offset;
						}
					}
					list_sender
						.send(ListMessage::TextureIndices(texture_indices))
						.expect("Failed to send texture index list");
				},
			);
		} else {
			self.texture_indices = Some(texture_indices);
		}

		self.device.poll(true);

		for _ in 0..n_updated_buffers {
			match list_receiver
				.recv()
				.expect("Failed to receive vertex/index list")
			{
				ListMessage::ColorVertices(list) => {
					self.color_vertices = Some(list)
				}
				ListMessage::ColorIndices(list) => {
					self.color_indices = Some(list)
				}
				ListMessage::TextureVertices(list) => {
					self.texture_vertices = Some(list)
				}
				ListMessage::TextureIndices(list) => {
					self.texture_indices = Some(list)
				}
			}
		}

		let zipped_iter = self
			.color_vertices
			.as_mut()
			.unwrap()
			.drain()
			.zip(self.color_indices.as_mut().unwrap().drain())
			.zip(self.texture_vertices.as_mut().unwrap().drain())
			.zip(self.texture_indices.as_mut().unwrap().drain())
			.enumerate()
			.map(|(i, (((c_v, c_i), t_v), t_i))| (i, c_v, c_i, t_v, t_i));

		for (
			i,
			color_vertex_buffer,
			color_index_buffer,
			texture_vertex_buffer,
			texture_index_buffer,
		) in zipped_iter
		{
			let buffers = DrawingBuffers {
				color_vertex_buffer,
				color_index_buffer,
				texture_vertex_buffer,
				texture_index_buffer,
			};
			self.drawing_manager.replace_buffers_at(i as u32, buffers);
		}

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
			for section in self
				.drawing_manager
				.text_queues()
				.flat_map(|queue| queue.sections.iter())
			{
				self.text_renderer.queue(section);
			}

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

trait BufferElement {
	type Offset: Copy;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorVertex {
	pos: ScreenNormPoint,
	color: Color,
}

impl ColorVertex {
	#[inline(always)]
	pub fn new(pos: ScreenNormPoint, color: Color) -> ColorVertex {
		ColorVertex { pos, color }
	}
}

unsafe impl AsBytes for ColorVertex {
	fn only_derive_is_allowed_to_implement_this_trait() {
		static_assertions::assert_eq_size!(ColorVertex, [f32; 5]);
	}
}

unsafe impl FromBytes for ColorVertex {
	fn only_derive_is_allowed_to_implement_this_trait() {
		static_assertions::assert_eq_size!(ColorVertex, [f32; 5]);
	}
}

impl BufferElement for ColorVertex {
	type Offset = ();
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextureVertex {
	pos: ScreenNormPoint,
	tex_coord: TexNormPoint,
}

impl TextureVertex {
	#[inline(always)]
	pub fn new(pos: ScreenNormPoint, tex_coord: TexNormPoint) -> TextureVertex {
		TextureVertex { pos, tex_coord }
	}
}

unsafe impl AsBytes for TextureVertex {
	fn only_derive_is_allowed_to_implement_this_trait() {
		static_assertions::assert_eq_size!(TextureVertex, [f32; 4]);
	}
}

unsafe impl FromBytes for TextureVertex {
	fn only_derive_is_allowed_to_implement_this_trait() {
		static_assertions::assert_eq_size!(TextureVertex, [f32; 4]);
	}
}

impl BufferElement for TextureVertex {
	type Offset = ();
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

impl BufferElement for VertexIndex {
	type Offset = VertexIndex;
}
