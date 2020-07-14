// Copyright 2020 Benjamin Scherer
// Licensed under the Open Software License version 3.0

use super::tessellation::GeometryBuilder;

use crate::{
	config::Config,
	geometry::{
		bounding_box_ext::BoundingBoxExt,
		ext::{ScreenPixelRectExt, TexPixelRectExt},
		ScreenPixelLength, ScreenPixelRect,
	},
	icons::{IconType, Icons},
	rendering::{
		color::Color, text::TextRenderer, ColorVertex, TextureVertex,
		VertexIndex, VertexIndexInner,
	},
	util::{Id, IdManager},
};

use bytemuck::{cast_slice, Pod};
use cfg_if::cfg_if;

use glyph_brush::{OwnedSection, Section};
use lyon_tessellation::{
	basic_shapes::{fill_rectangle, stroke_rectangle},
	FillOptions, StrokeOptions,
};
use winit::window::Window;

use std::{
	borrow::Cow,
	cmp::{Eq, PartialEq},
	convert::TryInto,
};

#[derive(Debug, Default)]
pub struct DrawingBuffers {
	pub color_vertex_buffer: Vec<ColorVertex>,
	pub color_index_buffer: Vec<VertexIndex<ColorVertex>>,
	pub texture_vertex_buffer: Vec<TextureVertex>,
	pub texture_index_buffer: Vec<VertexIndex<TextureVertex>>,
}

impl DrawingBuffers {
	fn clear(&mut self) {
		self.color_vertex_buffer.clear();
		self.color_index_buffer.clear();
		self.texture_vertex_buffer.clear();
		self.texture_index_buffer.clear();
	}
}

#[derive(Debug)]
enum DrawingBuffersEntry {
	Occupied(DrawingBuffers),
	Unoccupied(DrawingBuffers),
	Taken,
}

impl DrawingBuffersEntry {
	fn take_occupied(&mut self) -> Option<DrawingBuffers> {
		match self {
			DrawingBuffersEntry::Occupied(_) => {
				match std::mem::replace(self, DrawingBuffersEntry::Taken) {
					DrawingBuffersEntry::Occupied(buffers) => Some(buffers),
					_ => unreachable!(),
				}
			}
			_ => None,
		}
	}

	fn take_unoccupied(&mut self) -> Option<DrawingBuffers> {
		match self {
			DrawingBuffersEntry::Unoccupied(_) => {
				match std::mem::replace(self, DrawingBuffersEntry::Taken) {
					DrawingBuffersEntry::Unoccupied(buffers) => Some(buffers),
					_ => unreachable!(),
				}
			}
			_ => None,
		}
	}

	fn get_occupied_mut(&mut self) -> Option<&mut DrawingBuffers> {
		match self {
			DrawingBuffersEntry::Occupied(buffers) => Some(buffers),
			_ => None,
		}
	}

	fn get_occupied(&self) -> Option<&DrawingBuffers> {
		match self {
			DrawingBuffersEntry::Occupied(buffers) => Some(buffers),
			_ => None,
		}
	}

	fn clear(&mut self) {
		match self {
			DrawingBuffersEntry::Occupied(buffers) => buffers.clear(),
			_ => panic!("Attempted to clear taken or unoccupied buffers"),
		}
	}
}

#[derive(Debug, Default)]
pub struct TextQueue {
	pub sections: Vec<OwnedSection>,
}

impl TextQueue {
	fn queue<'a>(&mut self, section: impl Into<Cow<'a, Section<'a>>>) {
		self.sections.push(section.into().into_owned().to_owned());
	}

	fn clear(&mut self) {
		self.sections.clear();
	}
}

#[derive(Debug)]
pub struct BuffersEntry {
	text_queue: TextQueue,
	drawing_buffers: DrawingBuffersEntry,
}

impl BuffersEntry {
	fn clear(&mut self) {
		self.text_queue.clear();
		self.drawing_buffers.clear();
	}
}

#[derive(Debug, PartialEq, Eq)]
pub struct DrawingId(Id<u32>);

#[derive(Debug, Default)]
pub struct DrawingManager {
	buffers: Vec<BuffersEntry>,
	id_manager: IdManager<u32>,
}

impl DrawingManager {
	pub fn next_drawing_id(&mut self) -> DrawingId {
		for (i, entry) in self.buffers.iter_mut().enumerate() {
			let entry = &mut entry.drawing_buffers;
			if let Some(buffers) = entry.take_unoccupied() {
				*entry = DrawingBuffersEntry::Occupied(buffers);

				return DrawingId(self.id_manager.create_id(i as u32));
			}
		}

		self.buffers.push(BuffersEntry {
			text_queue: TextQueue::default(),
			drawing_buffers: DrawingBuffersEntry::Occupied(
				DrawingBuffers::default(),
			),
		});

		DrawingId(self.id_manager.create_id(self.buffers.len() as u32 - 1))
	}

	pub fn update(&mut self) {
		for relinquished_id in self.id_manager.reclaimed_ids() {
			let relinquished_id = relinquished_id as usize;
			match self.buffers.get_mut(relinquished_id).and_then(
				|buffer_entry| buffer_entry.drawing_buffers.take_occupied(),
			) {
				Some(mut buffers) => {
					buffers.clear();
					self.buffers[relinquished_id].drawing_buffers =
						DrawingBuffersEntry::Unoccupied(buffers)
				}
				None => log::error!(
					"Attempted to relinquish unoccupied or taken buffer"
				),
			}
		}
	}

	pub fn get_buffers_for(
		&mut self,
		id: &DrawingId,
	) -> Option<&mut DrawingBuffers> {
		let idx = *id.0 as usize;
		self.buffers.get_mut(idx).and_then(|buffer_entry| {
			buffer_entry.drawing_buffers.get_occupied_mut()
		})
	}

	pub fn get_text_queue_for(
		&mut self,
		id: &DrawingId,
	) -> Option<&mut TextQueue> {
		let idx = *id.0 as usize;
		self.buffers
			.get_mut(idx)
			.map(|buffer_entry| &mut buffer_entry.text_queue)
	}

	pub fn take_all_drawing_buffers(
		&mut self,
	) -> impl Iterator<Item = DrawingBuffers> + '_ {
		self.buffers.iter_mut().filter_map(|buffer_entry| {
			buffer_entry.drawing_buffers.take_occupied()
		})
	}

	pub fn drawing_buffers(
		&self,
	) -> impl Iterator<Item = &'_ DrawingBuffers> + '_ {
		self.buffers.iter().filter_map(|buffer_entry| {
			buffer_entry.drawing_buffers.get_occupied()
		})
	}

	pub fn element_iter<'buf, T: BufferElement<'buf> + Pod>(
		&'buf self,
	) -> T::Iter {
		T::Iter::new(self.buffers.as_slice())
	}

	pub fn text_queues(&self) -> impl Iterator<Item = &'_ TextQueue> + '_ {
		self.buffers
			.iter()
			.map(|buffer_entry| &buffer_entry.text_queue)
	}

	pub fn replace_buffers_at(&mut self, idx: u32, buffers: DrawingBuffers) {
		match self
			.buffers
			.get_mut(idx as usize)
			.map(|buffer_entry| &mut buffer_entry.drawing_buffers)
		{
			Some(entry @ DrawingBuffersEntry::Taken) => {
				*entry = DrawingBuffersEntry::Occupied(buffers)
			}
			Some(_) => log::error!(
				"Attempted to replace buffer {} that was not taken",
				idx
			),
			None => {
				log::error!("Attempted to replace nonexistent buffer {}", idx)
			}
		}
	}

	pub fn clear(&mut self, id: &DrawingId) {
		self.buffers
			.get_mut(*id.0 as usize)
			.expect("Attempted to clear nonexistent buffers")
			.clear();
	}
}

pub trait BufferElement<'buf>: Sized {
	type Iter: BufferElementIter<'buf, Self>;
}

impl<'buf> BufferElement<'buf> for ColorVertex {
	type Iter = BufferVertexElementIter<'buf, Self>;
}

impl<'buf> BufferElement<'buf> for TextureVertex {
	type Iter = BufferVertexElementIter<'buf, Self>;
}

pub trait BufferVertexElement: Sized {
	fn filter(buffers_entry: &BuffersEntry) -> Option<&[Self]>;
}

impl BufferVertexElement for ColorVertex {
	fn filter(buffers_entry: &BuffersEntry) -> Option<&[ColorVertex]> {
		buffers_entry
			.drawing_buffers
			.get_occupied()
			.map(|drawing_buffers| {
				drawing_buffers.color_vertex_buffer.as_slice()
			})
	}
}

impl BufferVertexElement for TextureVertex {
	fn filter(buffers_entry: &BuffersEntry) -> Option<&[TextureVertex]> {
		buffers_entry
			.drawing_buffers
			.get_occupied()
			.map(|drawing_buffers| {
				drawing_buffers.texture_vertex_buffer.as_slice()
			})
	}
}

pub trait BufferIndexElement<T: BufferVertexElement>: Sized {
	fn filter(buffers_entry: &BuffersEntry) -> Option<&[Self]>;
}

impl<'buf> BufferElement<'buf> for VertexIndex<ColorVertex> {
	type Iter =
		BufferIndexElementIter<'buf, ColorVertex, VertexIndex<ColorVertex>>;
}

impl<'buf> BufferElement<'buf> for VertexIndex<TextureVertex> {
	type Iter =
		BufferIndexElementIter<'buf, TextureVertex, VertexIndex<TextureVertex>>;
}

impl BufferIndexElement<ColorVertex> for VertexIndex<ColorVertex> {
	fn filter(
		buffers_entry: &BuffersEntry,
	) -> Option<&[VertexIndex<ColorVertex>]> {
		buffers_entry
			.drawing_buffers
			.get_occupied()
			.map(|drawing_buffers| {
				drawing_buffers.color_index_buffer.as_slice()
			})
	}
}

impl BufferIndexElement<TextureVertex> for VertexIndex<TextureVertex> {
	fn filter(
		buffers_entry: &BuffersEntry,
	) -> Option<&[VertexIndex<TextureVertex>]> {
		buffers_entry
			.drawing_buffers
			.get_occupied()
			.map(|drawing_buffers| {
				drawing_buffers.texture_index_buffer.as_slice()
			})
	}
}

pub trait BufferElementIter<'buf, T>: Iterator {
	fn new(buffers: &'buf [BuffersEntry]) -> Self;
	fn byte_len(&self) -> usize;
	fn total_len(&self) -> usize;
	fn is_empty(&self) -> bool;
}

type BufferVertexElementInnerIter<'buf, T> = std::iter::Scan<
	std::iter::FilterMap<
		std::slice::Iter<'buf, BuffersEntry>,
		fn(&'buf BuffersEntry) -> Option<&'buf [T]>,
	>,
	(usize, usize),
	for<'st> fn(
		&'st mut (usize, usize),
		&'buf [T],
	) -> Option<(usize, &'buf [T], usize)>,
>;

pub struct BufferVertexElementIter<'buf, T: BufferVertexElement> {
	buffers: &'buf [BuffersEntry],
	inner: BufferVertexElementInnerIter<'buf, T>,
}

impl<'buf, T: BufferVertexElement> Iterator
	for BufferVertexElementIter<'buf, T>
{
	type Item = (usize, &'buf [T], usize);

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next()
	}
}

impl<'buf, T: BufferVertexElement + Pod> BufferVertexElementIter<'buf, T> {
	fn scanner(
		(start, end): &mut (usize, usize),
		buffer: &'buf [T],
	) -> Option<(usize, &'buf [T], usize)> {
		*start = *end;
		*end += buffer.len();
		Some((*start, buffer, *end))
	}

	fn iter(
		buffers: &'buf [BuffersEntry],
	) -> BufferVertexElementInnerIter<'buf, T> {
		buffers
			.iter()
			.filter_map(T::filter as fn(_) -> _)
			.scan((0, 0), Self::scanner as for<'st> fn(&'st mut _, _) -> _)
	}
}

impl<'buf, T: BufferVertexElement + Pod> BufferElementIter<'buf, T>
	for BufferVertexElementIter<'buf, T>
{
	fn new(buffers: &'buf [BuffersEntry]) -> Self {
		Self {
			buffers,
			inner: Self::iter(buffers),
		}
	}

	fn byte_len(&self) -> usize {
		Self::iter(self.buffers)
			.map(|(_, slice, _)| cast_slice::<_, u8>(slice).len())
			.sum()
	}

	fn total_len(&self) -> usize {
		Self::iter(self.buffers)
			.map(|(_, slice, _)| slice.len())
			.sum()
	}

	fn is_empty(&self) -> bool {
		Self::iter(self.buffers).next().is_none()
	}
}

type BufferIndexElementInnerIter<'buf, V, I> = std::iter::Scan<
	std::iter::FilterMap<
		std::slice::Iter<'buf, BuffersEntry>,
		fn(&'buf BuffersEntry) -> Option<(&'buf [V], &'buf [I])>,
	>,
	(usize, usize, VertexIndexInner),
	for<'st> fn(
		&'st mut (usize, usize, VertexIndexInner),
		(&'buf [V], &'buf [I]),
	) -> Option<(usize, &'buf [I], usize, VertexIndexInner)>,
>;

pub struct BufferIndexElementIter<
	'buf,
	V: BufferVertexElement,
	I: BufferIndexElement<V>,
> {
	buffers: &'buf [BuffersEntry],
	inner: BufferIndexElementInnerIter<'buf, V, I>,
}

impl<'buf, V: BufferVertexElement, I: BufferIndexElement<V>> Iterator
	for BufferIndexElementIter<'buf, V, I>
{
	type Item = (usize, &'buf [I], usize, VertexIndexInner);

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next()
	}
}

impl<'buf, V: BufferVertexElement, I: BufferIndexElement<V> + Pod>
	BufferIndexElementIter<'buf, V, I>
{
	fn filter(buffer: &'buf BuffersEntry) -> Option<(&'buf [V], &'buf [I])> {
		V::filter(buffer).and_then(|v| I::filter(buffer).map(|i| (v, i)))
	}

	fn scanner(
		(start, end, offset): &mut (usize, usize, VertexIndexInner),
		(vertices, indices): (&'buf [V], &'buf [I]),
	) -> Option<(usize, &'buf [I], usize, VertexIndexInner)> {
		*start = *end;
		*end += indices.len();
		let res = Some((*start, indices, *end, *offset));
		*offset += {
			let len: VertexIndexInner = vertices.len().try_into().unwrap();
			len
		};
		res
	}

	fn iter(
		buffers: &'buf [BuffersEntry],
	) -> BufferIndexElementInnerIter<'buf, V, I> {
		buffers
			.iter()
			.filter_map(Self::filter as fn(_) -> _)
			.scan((0, 0, 0), Self::scanner as for<'st> fn(&'st mut _, _) -> _)
	}
}

impl<'buf, V: BufferVertexElement, I: BufferIndexElement<V> + Pod>
	BufferElementIter<'buf, I> for BufferIndexElementIter<'buf, V, I>
{
	fn new(buffers: &'buf [BuffersEntry]) -> Self {
		Self {
			buffers,
			inner: Self::iter(buffers),
		}
	}

	fn byte_len(&self) -> usize {
		Self::iter(self.buffers)
			.map(|(_, slice, _, _)| cast_slice::<_, u8>(slice).len())
			.sum()
	}

	fn total_len(&self) -> usize {
		Self::iter(self.buffers)
			.map(|(_, slice, _, _)| slice.len())
			.sum()
	}

	fn is_empty(&self) -> bool {
		Self::iter(self.buffers).next().is_none()
	}
}

pub struct DrawingContext<'a> {
	window: &'a Window,
	pub(super) config: &'a Config,
	icons: &'a Icons,
	manager: &'a mut DrawingManager,
	pub(super) text_renderer: &'a TextRenderer,
}

impl<'a> DrawingContext<'a> {
	pub fn new(
		window: &'a Window,
		config: &'a Config,
		icons: &'a Icons,
		manager: &'a mut DrawingManager,
		text_renderer: &'a TextRenderer,
	) -> DrawingContext<'a> {
		DrawingContext {
			window,
			config,
			icons,
			manager,
			text_renderer,
		}
	}

	pub(super) fn draw_solid_rect(
		&mut self,
		id: &DrawingId,
		bounding_box: ScreenPixelRect,
		color: Color,
	) {
		let buffers = match self.manager.get_buffers_for(id) {
			Some(buffers) => buffers,
			None => {
				cfg_if! {
					if #[cfg(debug_assertions)] {
						panic!("Could not get drawing buffers for {:?}", id);
					} else {
						log::error!("Could not get drawing buffers for {:?}", id);
						return;
					}
				}
			}
		};

		let window_size = self
			.window
			.inner_size()
			.to_logical(self.window.scale_factor());

		let mut builder = GeometryBuilder::new(
			&mut buffers.color_vertex_buffer,
			&mut buffers.color_index_buffer,
			color,
			window_size,
		);

		fill_rectangle(
			&bounding_box.cast_unit().to_f32(),
			&FillOptions::default(),
			&mut builder,
		)
		.expect("Failed to draw filled rectangle");
	}

	pub(super) fn draw_stroke_rect(
		&mut self,
		id: &DrawingId,
		bounding_box: ScreenPixelRect,
		stroke_width: ScreenPixelLength,
		color: Color,
	) {
		let buffers = match self.manager.get_buffers_for(id) {
			Some(buffers) => buffers,
			None => {
				cfg_if! {
					if #[cfg(debug_assertions)] {
						panic!("Could not get drawing buffers for {:?}", id);
					} else {
						log::error!("Could not get drawing buffers for {:?}", id);
						return;
					}
				}
			}
		};

		let window_size = self
			.window
			.inner_size()
			.to_logical(self.window.scale_factor());

		let mut builder = GeometryBuilder::new(
			&mut buffers.color_vertex_buffer,
			&mut buffers.color_index_buffer,
			color,
			window_size,
		);

		stroke_rectangle(
			&bounding_box.cast_unit().to_f32(),
			&StrokeOptions::default().with_line_width(stroke_width.0 as f32),
			&mut builder,
		)
		.expect("Failed to draw stroked rectangle");
	}

	pub(super) fn draw_icon_rect(
		&mut self,
		id: &DrawingId,
		bounding_box: ScreenPixelRect,
		icon: IconType,
	) {
		let buffers = match self.manager.get_buffers_for(id) {
			Some(buffers) => buffers,
			None => {
				cfg_if! {
					if #[cfg(debug_assertions)] {
						panic!("Could not get drawing buffers for {:?}", id);
					} else {
						log::error!("Could not get drawing buffers for {:?}", id);
						return;
					}
				}
			}
		};

		let window_size = self
			.window
			.inner_size()
			.to_logical(self.window.scale_factor());
		let start_idx: u16 = buffers
			.texture_vertex_buffer
			.len()
			.try_into()
			.expect("More than u16::MAX vertices");

		let bounding_box = bounding_box.to_norm(window_size);

		let icon_rect = self.icons.get_icon_descriptor(icon);

		let atlas_size = self.icons.texture_atlas_size();

		let icon_rect = icon_rect.to_norm(atlas_size);

		buffers.texture_vertex_buffer.extend_from_slice(&[
			TextureVertex::new(bounding_box.top_left(), icon_rect.top_left()),
			TextureVertex::new(bounding_box.top_right(), icon_rect.top_right()),
			TextureVertex::new(
				bounding_box.bottom_right(),
				icon_rect.bottom_right(),
			),
			TextureVertex::new(
				bounding_box.bottom_left(),
				icon_rect.bottom_left(),
			),
		]);

		buffers.texture_index_buffer.extend_from_slice(&[
			VertexIndex::from(0 + start_idx),
			VertexIndex::from(1 + start_idx),
			VertexIndex::from(2 + start_idx),
			VertexIndex::from(0 + start_idx),
			VertexIndex::from(3 + start_idx),
			VertexIndex::from(2 + start_idx),
		]);
	}

	pub(super) fn draw_text<'b>(
		&mut self,
		id: &DrawingId,
		section: impl Into<Cow<'b, Section<'b>>>,
	) {
		let text_queue = match self.manager.get_text_queue_for(id) {
			Some(queue) => queue,
			None => {
				cfg_if! {
					if #[cfg(debug_assertions)] {
						panic!("Could not get text queue for {:?}", id);
					} else {
						log::error!("Could not get text queue for {:?}", id);
						return;
					}
				}
			}
		};
		text_queue.queue(section);
	}

	pub(super) fn clear(&mut self, id: &DrawingId) {
		self.manager.clear(id);
	}
}
