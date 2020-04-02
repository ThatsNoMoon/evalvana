use super::{
	color::Color, point, text::TextRenderer, Point, Vertex, VertexIndex,
};

use crate::config::Config;
use crate::interface::{Interface, Pane, PaneList, Panes};
use crate::repl::evaluation::{
	CompoundResult, EditedExpression, Evaluation, Expression, PlainResult,
	Result as EvaluationResult,
};

use std::convert::TryInto;

use winit::{dpi::LogicalSize, window::Window};

pub struct DrawingContext<'a> {
	bounding_box: BoundingBox,
	window: &'a Window,
	config: &'a Config,
	vertex_buffer: &'a mut Vec<Vertex>,
	index_buffer: &'a mut Vec<VertexIndex>,
	text_renderer: &'a mut TextRenderer,
}

impl<'a> DrawingContext<'a> {
	pub fn new(
		window: &'a Window,
		config: &'a Config,
		vertex_buffer: &'a mut Vec<Vertex>,
		index_buffer: &'a mut Vec<VertexIndex>,
		text_renderer: &'a mut TextRenderer,
	) -> DrawingContext<'a> {
		let LogicalSize { width, height } =
			window.inner_size().to_logical(window.scale_factor());
		DrawingContext {
			bounding_box: BoundingBox::new(0, 0, width, height),
			window,
			config,
			vertex_buffer,
			index_buffer,
			text_renderer,
		}
	}

	fn with_bounding_box(
		&mut self,
		bounding_box: BoundingBox,
	) -> DrawingContext<'_> {
		DrawingContext {
			bounding_box,
			window: self.window,
			config: self.config,
			vertex_buffer: self.vertex_buffer,
			index_buffer: self.index_buffer,
			text_renderer: self.text_renderer,
		}
	}

	fn draw_solid_rect(&mut self, bounding_box: BoundingBox, color: Color) {
		let LogicalSize {
			width: w,
			height: h,
		} = self
			.window
			.inner_size()
			.to_logical(self.window.scale_factor());
		let start_idx: u16 = self
			.vertex_buffer
			.len()
			.try_into()
			.expect("More than u16::MAX vertices");
		self.vertex_buffer.extend_from_slice(&[
			Vertex::new(
				pixel_to_clip(bounding_box.x, bounding_box.y, w, h),
				color,
			),
			Vertex::new(
				pixel_to_clip(
					bounding_box.x + bounding_box.w,
					bounding_box.y,
					w,
					h,
				),
				color,
			),
			Vertex::new(
				pixel_to_clip(
					bounding_box.x + bounding_box.w,
					bounding_box.y + bounding_box.h,
					w,
					h,
				),
				color,
			),
			Vertex::new(
				pixel_to_clip(
					bounding_box.x,
					bounding_box.y + bounding_box.h,
					w,
					h,
				),
				color,
			),
		]);

		self.index_buffer.extend_from_slice(&[
			0 + start_idx,
			1 + start_idx,
			2 + start_idx,
			0 + start_idx,
			3 + start_idx,
			2 + start_idx,
		]);
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundingBox {
	x: u32,
	y: u32,
	w: u32,
	h: u32,
}

impl BoundingBox {
	fn new(x: u32, y: u32, w: u32, h: u32) -> BoundingBox {
		BoundingBox { x, y, w, h }
	}
}

pub trait Drawable {
	fn draw(&mut self, ctx: DrawingContext<'_>) -> BoundingBox;
}

impl Drawable for Interface {
	fn draw(&mut self, ctx: DrawingContext<'_>) -> BoundingBox {
		self.panes.draw(ctx)
	}
}

impl Drawable for Panes {
	fn draw(&mut self, mut ctx: DrawingContext<'_>) -> BoundingBox {
		match self {
			Panes::VerticalSplit(PaneList { panes, focused }) => {
				if panes.is_empty() {
					ctx.draw_solid_rect(
						ctx.bounding_box,
						ctx.config.ui_colors.bg,
					);
					return ctx.bounding_box;
				}
				let n: u32 = panes.len().try_into().unwrap();
				for (i, pane) in panes.iter_mut().enumerate() {
					let i: u32 = i.try_into().unwrap();
					let bounding_box = BoundingBox::new(
						ctx.bounding_box.x + ctx.bounding_box.w / n * i,
						ctx.bounding_box.y,
						ctx.bounding_box.w / n,
						ctx.bounding_box.h,
					);
					pane.draw(ctx.with_bounding_box(bounding_box));
				}
			}
			Panes::HorizontalSplit(PaneList { panes, focused }) => {
				if panes.is_empty() {
					ctx.draw_solid_rect(
						ctx.bounding_box,
						ctx.config.ui_colors.bg,
					);
					return ctx.bounding_box;
				}
				let n: u32 = panes.len().try_into().unwrap();
				for (i, pane) in panes.iter_mut().enumerate() {
					let i: u32 = i.try_into().unwrap();
					let bounding_box = BoundingBox::new(
						ctx.bounding_box.x,
						ctx.bounding_box.y + ctx.bounding_box.h / n * i,
						ctx.bounding_box.w,
						ctx.bounding_box.h / n,
					);
					pane.draw(ctx.with_bounding_box(bounding_box));
				}
			}
			Panes::Tabbed(PaneList { panes, focused }) => {
				if panes.is_empty() {
					ctx.draw_solid_rect(
						ctx.bounding_box,
						ctx.config.ui_colors.bg,
					);
					return ctx.bounding_box;
				}
			}
			_ => unimplemented!(),
		}
		ctx.bounding_box
	}
}

fn pixel_to_clip(x: u32, y: u32, w: u32, h: u32) -> Point {
	point(x as f32 / (w as f32) - 0.5, y as f32 / (h as f32) - 0.5)
}
