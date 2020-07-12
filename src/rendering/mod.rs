pub mod color;
use color::Color;

mod pipelines;
use pipelines::Pipelines;

mod text;
use text::TextRenderer;

pub mod drawing;
use drawing::{
	BufferElementIter, Drawable, DrawingContext, DrawingManager,
};

use crate::{
	config::Config,
	geometry::{
		ScreenNormPoint, ScreenPixelPoint, ScreenPixelRect, ScreenPixelSize,
		TexNormPoint,
	},
	icons::Icons,
	interface::Interface,
};

use std::{
	convert::{TryFrom, TryInto},
	marker::PhantomData,
	time::Instant,
};

use bytemuck::{cast_slice_mut, Pod, Zeroable};
use pollster::block_on;
use winit::{dpi::PhysicalSize, window::Window};

const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

pub struct Renderer {
	surface: wgpu::Surface,
	device: wgpu::Device,
	queue: wgpu::Queue,
	pipelines: Pipelines,
	swap_chain_descriptor: wgpu::SwapChainDescriptor,
	swap_chain: wgpu::SwapChain,

	text_renderer: TextRenderer,

	pub drawing_manager: DrawingManager,

	color_vertex_buffer: wgpu::Buffer,
	color_index_buffer: wgpu::Buffer,

	texture_vertex_buffer: wgpu::Buffer,
	texture_index_buffer: wgpu::Buffer,

	last_frame: Instant,
	delta_times: [f32; 20],
}

impl Renderer {
	pub async fn new(window: &Window, icons: &Icons) -> Renderer {
		let surface = wgpu::Surface::create(window);

		let adapter = wgpu::Adapter::request(
			&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::Default,
				compatible_surface: Some(&surface),
			},
			wgpu::BackendBit::PRIMARY,
		)
		.await
		.unwrap();

		let (mut device, queue) = adapter
			.request_device(&wgpu::DeviceDescriptor {
				extensions: wgpu::Extensions {
					anisotropic_filtering: false,
				},
				limits: wgpu::Limits::default(),
			})
			.await;

		let text_renderer = TextRenderer::new(&mut device, COLOR_FORMAT);

		let pipelines = Pipelines::new(&device, icons.texture_atlas_size());

		{
			let mut encoder = device.create_command_encoder(
				&wgpu::CommandEncoderDescriptor {
					label: Some("evalvana_icon_fill_encoder"),
				},
			);

			icons.fill_texture_atlas(
				&device,
				pipelines.texture_atlas(),
				&mut encoder,
			);

			queue.submit(&[encoder.finish()]);
		}

		device.poll(wgpu::Maintain::Wait);

		let color_vertex_buffer =
			device.create_buffer(&wgpu::BufferDescriptor {
				label: Some("evalvana_color_vertex_buffer"),
				size: 500
					* std::mem::size_of::<ColorVertex>() as wgpu::BufferAddress,
				usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::MAP_WRITE,
			});

		let color_index_buffer =
			device.create_buffer(&wgpu::BufferDescriptor {
				label: Some("evalvana_color_index_buffer"),
				size: 500
					* std::mem::size_of::<VertexIndex<ColorVertex>>()
						as wgpu::BufferAddress,
				usage: wgpu::BufferUsage::INDEX | wgpu::BufferUsage::MAP_WRITE,
			});

		let texture_vertex_buffer =
			device.create_buffer(&wgpu::BufferDescriptor {
				label: Some("evalvana_texture_vertex_buffer"),
				size: 500
					* std::mem::size_of::<TextureVertex>()
						as wgpu::BufferAddress,
				usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::MAP_WRITE,
			});

		let texture_index_buffer =
			device.create_buffer(&wgpu::BufferDescriptor {
				label: Some("evalvana_texture_index_buffer"),
				size: 500
					* std::mem::size_of::<VertexIndex<TextureVertex>>()
						as wgpu::BufferAddress,
				usage: wgpu::BufferUsage::INDEX | wgpu::BufferUsage::MAP_WRITE,
			});

		let size = window.inner_size();
		let sc_desc = wgpu::SwapChainDescriptor {
			usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
			format: COLOR_FORMAT,
			width: size.width,
			height: size.height,
			present_mode: wgpu::PresentMode::Fifo,
		};

		let swap_chain = device.create_swap_chain(&surface, &sc_desc);

		let last_frame = Instant::now();

		let delta_times = [0f32; 20];

		let drawing_manager = DrawingManager::default();

		Renderer {
			surface,
			device,
			queue,
			pipelines,
			swap_chain_descriptor: sc_desc,
			swap_chain,

			text_renderer,

			drawing_manager,

			color_vertex_buffer,
			color_index_buffer,

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
		log::trace!("Redrawing");

		{
			log::trace!("Drawing interface");

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

		let n_color_indices = self
			.drawing_manager
			.element_iter::<VertexIndex<ColorVertex>>()
			.total_len() as u32;
		let n_texture_indices = self
			.drawing_manager
			.element_iter::<VertexIndex<TextureVertex>>()
			.total_len() as u32;
		log::trace!("Mapping buffers");

		let color_vertices_fut = {
			let color_vertices =
				self.drawing_manager.element_iter::<ColorVertex>();
			let color_vertex_buffer = &self.color_vertex_buffer;
			let buffer_map_future = color_vertex_buffer.map_write(0, color_vertices.byte_len() as wgpu::BufferAddress);
			async move {
				if !color_vertices.is_empty() {
					let mut mapped_buffer = buffer_map_future
						.await
						.expect("Failed to map color vertex buffer");

					log::trace!("Writing to mapped color vertex buffer");

					let dest = cast_slice_mut::<_, ColorVertex>(mapped_buffer.as_slice());

					for (start, src, end) in color_vertices {
						dest[start..end].copy_from_slice(src);
					}
				}
			}
		};

		let color_indices_fut = {
			let color_indices = self
				.drawing_manager
				.element_iter::<VertexIndex<ColorVertex>>();
			let color_index_buffer = &self.color_index_buffer;
			let buffer_map_future = color_index_buffer.map_write(0, color_indices.byte_len() as wgpu::BufferAddress);
			async move {
				if !color_indices.is_empty() {
					let mut mapped_buffer = buffer_map_future
						.await
						.expect("Failed to map color index buffer");

					log::trace!("Writing to mapped color index buffer");

					let dest = cast_slice_mut::<_, VertexIndex<ColorVertex>>(
						mapped_buffer.as_slice(),
					);

					for (start, src, end, offset) in color_indices {
						dest[start..end].copy_from_slice(src);
						for index in dest[start..end].iter_mut() {
							index.inner += offset;
						}
					}
				}
			}
		};

		let texture_vertices_fut = {
			let texture_vertices =
				self.drawing_manager.element_iter::<TextureVertex>();
			let texture_vertex_buffer = &self.texture_vertex_buffer;
			let buffer_map_future = texture_vertex_buffer.map_write(0, texture_vertices.byte_len() as wgpu::BufferAddress);
			async move {
				if !texture_vertices.is_empty() {
					let mut mapped_buffer = buffer_map_future
						.await
						.expect("Failed to map texture vertex buffer");

					log::trace!("Writing to mapped texture vertex buffer");

					let dest = cast_slice_mut::<_, TextureVertex>(mapped_buffer.as_slice());

					for (start, src, end) in texture_vertices {
						dest[start..end].copy_from_slice(src);
					}
				}
			}
		};

		let texture_indices_fut = {
			let texture_indices = self
				.drawing_manager
				.element_iter::<VertexIndex<TextureVertex>>();
			let texture_index_buffer = &self.texture_index_buffer;
			let buffer_map_future = texture_index_buffer.map_write(0, texture_indices.byte_len() as wgpu::BufferAddress);
			async move {
				if !texture_indices.is_empty() {
					let mut mapped_buffer = buffer_map_future
						.await
						.expect("Failed to map texture index buffer");

					log::trace!("Writing to mapped texture index buffer");

					let dest = cast_slice_mut::<_, VertexIndex<TextureVertex>>(
						mapped_buffer.as_slice(),
					);

					for (start, src, end, offset) in texture_indices {
						dest[start..end].copy_from_slice(src);
						for index in dest[start..end].iter_mut() {
							index.inner += offset;
						}
					}
				}
			}
		};
		log::trace!("Polling device");

		self.device.poll(wgpu::Maintain::Wait);

		log::trace!("Awaiting futures");

		block_on(async {
			color_vertices_fut.await;
			color_indices_fut.await;
			texture_vertices_fut.await;
			texture_indices_fut.await;
		});

		let frame = self
			.swap_chain
			.get_next_texture()
			.expect("Failed to get next texture from swapchain");
		let mut encoder = self.device.create_command_encoder(
			&wgpu::CommandEncoderDescriptor {
				label: Some("evalvana_drawing_encoder"),
			},
		);

		log::trace!("Creating render passes");

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
			rpass.set_vertex_buffer(0, &self.color_vertex_buffer, 0, 0);
			rpass.set_index_buffer(&self.color_index_buffer, 0, 0);
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
			rpass.set_vertex_buffer(0, &self.texture_vertex_buffer, 0, 0);
			rpass.set_index_buffer(&self.texture_index_buffer, 0, 0);
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

static_assertions::assert_eq_size!(ColorVertex, [f32; 5]);
unsafe impl Zeroable for ColorVertex {}

unsafe impl Pod for ColorVertex {}

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

static_assertions::assert_eq_size!(TextureVertex, [f32; 4]);
unsafe impl Zeroable for TextureVertex {}

unsafe impl Pod for TextureVertex {}

pub type VertexIndexInner = u16;
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VertexIndex<T> {
	pub inner: VertexIndexInner,
	_marker: PhantomData<T>,
}

static_assertions::assert_eq_size!(VertexIndex<ColorVertex>, u16);
static_assertions::assert_eq_size!(VertexIndex<TextureVertex>, u16);
unsafe impl<T> Zeroable for VertexIndex<T> {}

unsafe impl<T: Copy + 'static> Pod for VertexIndex<T> {}

impl<T> From<VertexIndexInner> for VertexIndex<T> {
	fn from(idx: VertexIndexInner) -> Self {
		Self {
			inner: idx,
			_marker: PhantomData::default(),
		}
	}
}

impl<T> TryFrom<u32> for VertexIndex<T> {
	type Error = <VertexIndexInner as TryFrom<u32>>::Error;

	fn try_from(idx: u32) -> Result<Self, Self::Error> {
		idx.try_into().map(|idx| Self {
			inner: idx,
			_marker: PhantomData::default(),
		})
	}
}

impl<T> TryFrom<usize> for VertexIndex<T> {
	type Error = <VertexIndexInner as TryFrom<usize>>::Error;

	fn try_from(idx: usize) -> Result<Self, Self::Error> {
		idx.try_into().map(|idx| Self {
			inner: idx,
			_marker: PhantomData::default(),
		})
	}
}

impl<T> VertexIndex<T> {
	pub fn to_u32(&self) -> u32 {
		self.inner as u32
	}

	pub fn to_usize(&self) -> usize {
		self.inner as usize
	}
}

trait VertexIndexFormat {
	const WGPU_FORMAT: wgpu::IndexFormat;
}

impl<T> VertexIndexFormat for VertexIndex<T> {
	const WGPU_FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint16;
}
