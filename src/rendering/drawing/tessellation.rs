use crate::{
	geometry::{ext::ScreenPixelPointExt, ScreenPixelPoint},
	rendering::{color::Color, ColorVertex, VertexIndex},
};

use lyon_tessellation::{
	math::Point as LyonPoint, BasicGeometryBuilder as LyonBasicGeometryBuilder,
	Count, FillAttributes, FillGeometryBuilder as LyonFillGeometryBuilder,
	GeometryBuilder as LyonGeometryBuilder,
	GeometryBuilderError as LyonGeometryBuilderError, StrokeAttributes,
	StrokeGeometryBuilder as LyonStrokeGeometryBuilder,
	VertexId as LyonVertexId,
};
use winit::dpi::LogicalSize;

pub(super) struct GeometryBuilder<'a> {
	vertices: &'a mut Vec<ColorVertex>,
	indices: &'a mut Vec<VertexIndex>,
	vertex_offset: VertexIndex,
	index_offset: VertexIndex,

	color: Color,
	window_size: LogicalSize<u32>,
}

impl GeometryBuilder<'_> {
	pub(super) fn new<'a>(
		vertices: &'a mut Vec<ColorVertex>,
		indices: &'a mut Vec<VertexIndex>,
		color: Color,
		window_size: LogicalSize<u32>,
	) -> GeometryBuilder<'a> {
		GeometryBuilder {
			vertex_offset: vertices.len() as VertexIndex,
			index_offset: indices.len() as VertexIndex,
			vertices,
			indices,
			color,
			window_size,
		}
	}
}

impl<'a> LyonGeometryBuilder for GeometryBuilder<'a> {
	fn begin_geometry(&mut self) {
		self.vertex_offset = self.vertices.len() as VertexIndex;
		self.index_offset = self.indices.len() as VertexIndex;
	}

	fn end_geometry(&mut self) -> Count {
		Count {
			vertices: self.vertices.len() as u32 - self.vertex_offset as u32,
			indices: self.indices.len() as u32 - self.index_offset as u32,
		}
	}

	fn abort_geometry(&mut self) {
		self.vertices.truncate(self.vertex_offset as usize);
		self.indices.truncate(self.index_offset as usize);
	}

	fn add_triangle(
		&mut self,
		a: LyonVertexId,
		b: LyonVertexId,
		c: LyonVertexId,
	) {
		debug_assert!(a != b);
		debug_assert!(a != c);
		debug_assert!(b != c);
		debug_assert!(a != LyonVertexId::INVALID);
		debug_assert!(b != LyonVertexId::INVALID);
		debug_assert!(c != LyonVertexId::INVALID);

		self.indices.push((a + self.vertex_offset as u32).into());
		self.indices.push((b + self.vertex_offset as u32).into());
		self.indices.push((c + self.vertex_offset as u32).into());
	}
}

impl<'a> LyonBasicGeometryBuilder for GeometryBuilder<'a> {
	fn add_vertex(
		&mut self,
		pos: LyonPoint,
	) -> Result<LyonVertexId, LyonGeometryBuilderError> {
		let pos: ScreenPixelPoint = pos.cast_unit().round().to_u32();
		let pos = pos.to_norm(self.window_size);
		self.vertices.push(ColorVertex::new(pos, self.color));

		let len = self.vertices.len();
		if len > VertexIndex::max_value() as usize {
			Err(LyonGeometryBuilderError::TooManyVertices)
		} else {
			Ok(((len - 1) as VertexIndex - self.vertex_offset).into())
		}
	}
}

impl<'a> LyonStrokeGeometryBuilder for GeometryBuilder<'a> {
	fn add_stroke_vertex(
		&mut self,
		pos: LyonPoint,
		_: StrokeAttributes,
	) -> Result<LyonVertexId, LyonGeometryBuilderError> {
		self.add_vertex(pos)
	}
}

impl<'a> LyonFillGeometryBuilder for GeometryBuilder<'a> {
	fn add_fill_vertex(
		&mut self,
		pos: LyonPoint,
		_: FillAttributes,
	) -> Result<LyonVertexId, LyonGeometryBuilderError> {
		self.add_vertex(pos)
	}
}
