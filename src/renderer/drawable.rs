use super::{
	bounding_box::BoundingBox, color::Color, point, text::TextRenderer, Point,
	Vertex, VertexIndex,
};

use crate::config::Config;
use crate::interface::{
	Evaluator, Evaluators, Interface, Pane, PaneList, PaneStatus,
	PaneStatusList, PaneStatuses, Panes, TreePane,
};
use crate::repl::evaluation::{
	CompoundResult, EditedExpression, Evaluation, Expression, PlainResult,
	Result as EvaluationResult,
};

use std::borrow::Cow;
use std::convert::TryInto;

use wgpu_glyph::{Scale as FontScale, Section, VariedSection};
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

	fn reborrow(&mut self) -> DrawingContext<'_> {
		DrawingContext {
			bounding_box: self.bounding_box,
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

	fn draw_text<'b>(
		&mut self,
		section: impl Into<Cow<'b, VariedSection<'b>>>,
	) {
		self.text_renderer.queue(section);
	}
}

pub trait Drawable {
	fn draw(&mut self, ctx: DrawingContext<'_>) -> BoundingBox;
}

impl Drawable for Interface {
	fn draw(&mut self, mut ctx: DrawingContext<'_>) -> BoundingBox {
		let mut tree_bounding_box = ctx.bounding_box.with_w(225);
		tree_bounding_box = self
			.tree_pane
			.draw(ctx.with_bounding_box(tree_bounding_box));
		let mut panes_bounding_box =
			ctx.bounding_box.added_left(tree_bounding_box.w);
		panes_bounding_box =
			self.panes.draw(ctx.with_bounding_box(panes_bounding_box));

		tree_bounding_box.added_w(panes_bounding_box.w)
	}
}

impl Drawable for TreePane {
	fn draw(&mut self, mut ctx: DrawingContext<'_>) -> BoundingBox {
		ctx.draw_solid_rect(
			ctx.bounding_box,
			ctx.config.ui_colors.secondary_bg,
		);

		let mut current_bounding_box = ctx.bounding_box.added_top(8);

		let statuses_title_bounding_box =
			current_bounding_box.added_left(10).with_h(20);

		let statuses_title = Section {
			text: "Open REPLs",
			color: ctx.config.ui_colors.accent.to_rgba(),
			font_id: ctx.text_renderer.ui_font_medium(),
			scale: FontScale::uniform(20.0),
			..statuses_title_bounding_box.to_section_bounds()
		};
		ctx.draw_text(statuses_title);

		current_bounding_box = current_bounding_box
			.added_top(statuses_title_bounding_box.h)
			.added_top(10);

		let statuses_bounding_box = self
			.pane_statuses
			.draw(ctx.with_bounding_box(current_bounding_box));

		current_bounding_box = current_bounding_box
			.added_top(statuses_bounding_box.h)
			.added_top(40);

		let evaluators_title_bounding_box =
			current_bounding_box.with_h(20).added_left(10);

		let evaluators_title = Section {
			text: "Available REPLs",
			color: ctx.config.ui_colors.accent.to_rgba(),
			font_id: ctx.text_renderer.ui_font_medium(),
			scale: FontScale::uniform(20.0),
			..evaluators_title_bounding_box.to_section_bounds()
		};
		ctx.draw_text(evaluators_title);

		current_bounding_box = current_bounding_box
			.added_top(evaluators_title_bounding_box.h)
			.added_top(10);

		self.evaluators
			.draw(ctx.with_bounding_box(current_bounding_box));
		ctx.bounding_box
	}
}

impl Drawable for PaneStatuses {
	fn draw(&mut self, mut ctx: DrawingContext<'_>) -> BoundingBox {
		let pane_status_list = match self {
			PaneStatuses::VerticalSplit(list) => list,
			PaneStatuses::HorizontalSplit(list) => list,
			PaneStatuses::Tabbed(list) => list,

			PaneStatuses::Single(status) => return status.draw(ctx),
		};

		if pane_status_list.pane_statuses.is_empty() {
			return ctx.bounding_box.with_h(0);
		}

		pane_status_list.draw(ctx.reborrow())
	}
}

impl Drawable for PaneStatusList {
	fn draw(&mut self, mut ctx: DrawingContext<'_>) -> BoundingBox {
		let mut current_bounding_box = ctx.bounding_box;
		let mut drawn_bounding_box = ctx.bounding_box.with_h(0);

		for inner_statuses in &mut self.pane_statuses {
			let inner_bounding_box = inner_statuses
				.draw(ctx.with_bounding_box(current_bounding_box));

			current_bounding_box =
				current_bounding_box.added_top(inner_bounding_box.h);
			drawn_bounding_box =
				drawn_bounding_box.with_bottom(inner_bounding_box.bottom());
		}
		ctx.bounding_box.with_h(drawn_bounding_box.h)
	}
}

impl Drawable for PaneStatus {
	fn draw(&mut self, mut ctx: DrawingContext<'_>) -> BoundingBox {
		let bounding_box = ctx.bounding_box.with_h(24).added_left(20);
		ctx.draw_text(Section {
			text: self.name.as_str(),
			color: ctx.config.ui_colors.text.to_rgba(),
			font_id: ctx.text_renderer.ui_font(),
			..bounding_box.to_section_bounds()
		});
		bounding_box
	}
}

impl Drawable for Evaluators {
	fn draw(&mut self, mut ctx: DrawingContext<'_>) -> BoundingBox {
		let mut current_bounding_box = ctx.bounding_box;
		let mut drawn_bounding_box = ctx.bounding_box.with_h(0);

		for evaluator in &mut self.evaluators {
			let mut inner_bounding_box = current_bounding_box.with_h(75);

			inner_bounding_box =
				evaluator.draw(ctx.with_bounding_box(inner_bounding_box));

			current_bounding_box =
				current_bounding_box.added_top(inner_bounding_box.h);
			drawn_bounding_box =
				drawn_bounding_box.with_bottom(inner_bounding_box.bottom());
		}
		ctx.bounding_box.with_h(drawn_bounding_box.h)
	}
}

impl Drawable for Evaluator {
	fn draw(&mut self, mut ctx: DrawingContext<'_>) -> BoundingBox {
		ctx.draw_solid_rect(
			ctx.bounding_box
				.added_left(13)
				.subbed_right(13)
				.added_top(7)
				.subbed_bottom(7),
			ctx.config.ui_colors.bg,
		);

		ctx.draw_text(Section {
			text: self.name.as_str(),
			color: ctx.config.ui_colors.text.to_rgba(),
			font_id: ctx.text_renderer.ui_font_medium(),
			layout: wgpu_glyph::Layout::default()
				.v_align(wgpu_glyph::VerticalAlign::Center),
			..ctx
				.bounding_box
				.added_x(22)
				.added_y(ctx.bounding_box.h / 2)
				.with_h(ctx.bounding_box.h / 3)
				.to_section_bounds()
		});
		ctx.bounding_box
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
	point(
		(x as f32 / (w as f32) - 0.5) * 2.0,
		(y as f32 / (h as f32) - 0.5) * 2.0,
	)
}
