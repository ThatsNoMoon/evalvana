use super::{
	bounding_box::BoundingBox, color::Color, text::TextRenderer, Point,
	TextureVertex, Vertex, VertexIndex,
};

use crate::{
	config::Config,
	icons::{IconDescriptor, IconType, Icons},
	interface::{
		Evaluator, Evaluators, Interface, Pane, PaneList, PaneStatus,
		PaneStatuses, Panes, TreePane,
	},
	repl::evaluation::{
		CompoundResult, EditedExpression, Evaluation, Expression, PlainResult,
		Result as EvaluationResult,
	},
};

use std::borrow::Cow;
use std::convert::TryInto;

use wgpu_glyph::{
	Layout, Scale as FontScale, Section, VariedSection, VerticalAlign,
};
use winit::{dpi::LogicalSize, window::Window};

pub struct DrawingContext<'a> {
	bounding_box: BoundingBox,
	window: &'a Window,
	config: &'a Config,
	icons: &'a Icons,
	color_vertex_buffer: &'a mut Vec<Vertex>,
	color_index_buffer: &'a mut Vec<VertexIndex>,
	texture_vertex_buffer: &'a mut Vec<TextureVertex>,
	texture_index_buffer: &'a mut Vec<VertexIndex>,
	text_renderer: &'a mut TextRenderer,
}

impl<'a> DrawingContext<'a> {
	pub fn new(
		window: &'a Window,
		config: &'a Config,
		icons: &'a Icons,
		color_vertex_buffer: &'a mut Vec<Vertex>,
		color_index_buffer: &'a mut Vec<VertexIndex>,
		texture_vertex_buffer: &'a mut Vec<TextureVertex>,
		texture_index_buffer: &'a mut Vec<VertexIndex>,
		text_renderer: &'a mut TextRenderer,
	) -> DrawingContext<'a> {
		let LogicalSize { width, height } =
			window.inner_size().to_logical(window.scale_factor());
		DrawingContext {
			bounding_box: BoundingBox::new(0, 0, width, height),
			window,
			config,
			icons,
			color_vertex_buffer,
			color_index_buffer,
			texture_vertex_buffer,
			texture_index_buffer,
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
			icons: self.icons,
			color_vertex_buffer: self.color_vertex_buffer,
			color_index_buffer: self.color_index_buffer,
			texture_vertex_buffer: self.texture_vertex_buffer,
			texture_index_buffer: self.texture_index_buffer,
			text_renderer: self.text_renderer,
		}
	}

	fn reborrow(&mut self) -> DrawingContext<'_> {
		DrawingContext {
			bounding_box: self.bounding_box,
			window: self.window,
			config: self.config,
			icons: self.icons,
			color_vertex_buffer: self.color_vertex_buffer,
			color_index_buffer: self.color_index_buffer,
			texture_vertex_buffer: self.texture_vertex_buffer,
			texture_index_buffer: self.texture_index_buffer,
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
			.color_vertex_buffer
			.len()
			.try_into()
			.expect("More than u16::MAX vertices");
		self.color_vertex_buffer.extend_from_slice(&[
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

		self.color_index_buffer.extend_from_slice(&[
			0 + start_idx,
			1 + start_idx,
			2 + start_idx,
			0 + start_idx,
			3 + start_idx,
			2 + start_idx,
		]);
	}

	fn draw_icon_rect(&mut self, bounding_box: BoundingBox, icon: IconType) {
		let LogicalSize {
			width: w,
			height: h,
		} = self
			.window
			.inner_size()
			.to_logical(self.window.scale_factor());
		let start_idx: u16 = self
			.texture_vertex_buffer
			.len()
			.try_into()
			.expect("More than u16::MAX vertices");

		let IconDescriptor {
			size: (icon_w, icon_h),
			atlas_location: (icon_x, icon_y),
		} = self.icons.get_icon_descriptor(icon);

		let (atlas_w, atlas_h) = self.icons.texture_atlas_size();
		let (atlas_w, atlas_h) = (atlas_w as f32, atlas_h as f32);

		self.texture_vertex_buffer.extend_from_slice(&[
			TextureVertex::new(
				pixel_to_clip(bounding_box.x, bounding_box.y, w, h),
				Point::new(icon_x as f32, icon_y as f32),
			),
			TextureVertex::new(
				pixel_to_clip(
					bounding_box.x + bounding_box.w,
					bounding_box.y,
					w,
					h,
				),
				Point::new(
					(icon_x + icon_w - 1) as f32 / atlas_w,
					icon_y as f32 / atlas_h,
				),
			),
			TextureVertex::new(
				pixel_to_clip(
					bounding_box.x + bounding_box.w,
					bounding_box.y + bounding_box.h,
					w,
					h,
				),
				Point::new(
					(icon_x + icon_w - 1) as f32 / atlas_w,
					(icon_y + icon_h - 1) as f32 / atlas_h,
				),
			),
			TextureVertex::new(
				pixel_to_clip(
					bounding_box.x,
					bounding_box.y + bounding_box.h,
					w,
					h,
				),
				Point::new(
					icon_x as f32 / atlas_w,
					(icon_y + icon_h - 1) as f32 / atlas_h,
				),
			),
		]);

		self.texture_index_buffer.extend_from_slice(&[
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
	type UserData;
	fn draw(
		&mut self,
		ctx: DrawingContext<'_>,
		data: Self::UserData,
	) -> BoundingBox;
}

impl Drawable for Interface {
	type UserData = ();

	fn draw(&mut self, mut ctx: DrawingContext<'_>, _: ()) -> BoundingBox {
		let mut tree_bounding_box = ctx.bounding_box.with_w(225);
		tree_bounding_box = self
			.tree_pane
			.draw(ctx.with_bounding_box(tree_bounding_box), ());
		let mut panes_bounding_box =
			ctx.bounding_box.added_left(tree_bounding_box.w);
		panes_bounding_box = self
			.panes
			.draw(ctx.with_bounding_box(panes_bounding_box), true);

		tree_bounding_box.added_w(panes_bounding_box.w)
	}
}

impl Drawable for TreePane {
	type UserData = ();

	fn draw(&mut self, mut ctx: DrawingContext<'_>, _: ()) -> BoundingBox {
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
			.draw(ctx.with_bounding_box(current_bounding_box), ());

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
			.draw(ctx.with_bounding_box(current_bounding_box), ());
		ctx.bounding_box
	}
}

impl Drawable for PaneStatuses {
	type UserData = ();

	fn draw(&mut self, mut ctx: DrawingContext<'_>, _: ()) -> BoundingBox {
		let mut current_bounding_box = ctx.bounding_box;
		let mut drawn_bounding_box = ctx.bounding_box.with_h(0);

		for (i, pane_status) in self.pane_statuses.iter_mut().enumerate() {
			let i: u32 = i.try_into().unwrap();
			let mut inner_bounding_box = current_bounding_box.with_h(24);

			inner_bounding_box = pane_status.draw(
				ctx.with_bounding_box(inner_bounding_box),
				i == self.focused,
			);

			if drawn_bounding_box.bottom() + inner_bounding_box.h
				>= ctx.bounding_box.bottom()
			{
				break;
			}

			current_bounding_box =
				current_bounding_box.added_top(inner_bounding_box.h);
			drawn_bounding_box =
				drawn_bounding_box.with_bottom(inner_bounding_box.bottom());
		}
		ctx.bounding_box.with_h(drawn_bounding_box.h)
	}
}

impl Drawable for PaneStatus {
	type UserData = bool;

	fn draw(
		&mut self,
		mut ctx: DrawingContext<'_>,
		is_focused: bool,
	) -> BoundingBox {
		if is_focused {
			ctx.draw_solid_rect(
				ctx.bounding_box,
				ctx.config.ui_colors.focused_bg,
			);
		}
		let text_bounding_box = ctx
			.bounding_box
			.added_left(20)
			.added_y(ctx.bounding_box.h / 2)
			.added_right(ctx.bounding_box.h);
		ctx.draw_text(Section {
			text: self.name.as_str(),
			color: ctx.config.ui_colors.text.to_rgba(),
			font_id: ctx.text_renderer.ui_font(),
			layout: Layout::default().v_align(VerticalAlign::Center),
			..text_bounding_box.to_section_bounds()
		});

		let icon_margin = u32::max(2, ctx.bounding_box.h / 5);
		let icon_bounding_box = ctx
			.bounding_box
			.with_w(ctx.bounding_box.h)
			.added_x(ctx.bounding_box.w - ctx.bounding_box.h)
			.added_left(icon_margin)
			.subbed_right(icon_margin)
			.added_top(icon_margin)
			.subbed_bottom(icon_margin);

		ctx.draw_icon_rect(icon_bounding_box, IconType::Close);

		ctx.bounding_box
	}
}

impl Drawable for Evaluators {
	type UserData = ();

	fn draw(&mut self, mut ctx: DrawingContext<'_>, _: ()) -> BoundingBox {
		let mut current_bounding_box = ctx.bounding_box;
		let mut drawn_bounding_box = ctx.bounding_box.with_h(0);

		for evaluator in &mut self.evaluators {
			let mut inner_bounding_box = current_bounding_box.with_h(75);

			inner_bounding_box =
				evaluator.draw(ctx.with_bounding_box(inner_bounding_box), ());

			if drawn_bounding_box.bottom() + inner_bounding_box.h
				>= ctx.bounding_box.bottom()
			{
				break;
			}

			current_bounding_box =
				current_bounding_box.added_top(inner_bounding_box.h);
			drawn_bounding_box =
				drawn_bounding_box.with_bottom(inner_bounding_box.bottom());
		}
		ctx.bounding_box.with_h(drawn_bounding_box.h)
	}
}

impl Drawable for Evaluator {
	type UserData = ();

	fn draw(&mut self, mut ctx: DrawingContext<'_>, _: ()) -> BoundingBox {
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
			layout: Layout::default().v_align(VerticalAlign::Center),
			..ctx
				.bounding_box
				.added_left(22)
				.added_y(ctx.bounding_box.h / 2)
				.with_h(ctx.bounding_box.h / 3)
				.to_section_bounds()
		});
		ctx.bounding_box
	}
}

impl Drawable for Panes {
	type UserData = bool;

	fn draw(
		&mut self,
		mut ctx: DrawingContext<'_>,
		is_focused: bool,
	) -> BoundingBox {
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
				let inner_bounding_box = {
					let total_width = ctx.bounding_box.w - 2 * (n - 1);
					let total_width = total_width as f64;
					let inner_pane_width = total_width / (n as f64);
					let inner_pane_width = inner_pane_width.ceil() as u32;
					ctx.bounding_box.with_w(inner_pane_width)
				};

				for (i, pane) in panes.iter_mut().enumerate() {
					let i: u32 = i.try_into().unwrap();
					pane.draw(
						ctx.with_bounding_box(
							inner_bounding_box
								.added_x((inner_bounding_box.w + 2) * i),
						),
						is_focused && i == *focused,
					);
					ctx.draw_solid_rect(
						inner_bounding_box
							.added_x(inner_bounding_box.w)
							.with_w(2),
						ctx.config.ui_colors.borders,
					);
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

				let inner_bounding_box =
					ctx.bounding_box.with_h(ctx.bounding_box.h / n);

				for (i, pane) in panes.iter_mut().enumerate() {
					let i: u32 = i.try_into().unwrap();
					pane.draw(
						ctx.with_bounding_box(
							inner_bounding_box
								.added_y(inner_bounding_box.h * i),
						),
						is_focused && i == *focused,
					);
				}
			}
			Panes::Tabbed(PaneList { panes, focused }) => {
				if panes.is_empty() {
					ctx.draw_solid_rect(
						ctx.bounding_box,
						ctx.config.ui_colors.borders,
					);
					return ctx.bounding_box;
				}

				let tabs_bg_bounding_box = ctx.bounding_box.with_h(40);

				ctx.draw_solid_rect(
					tabs_bg_bounding_box,
					ctx.config.ui_colors.borders,
				);

				fn draw_tab(
					mut ctx: DrawingContext<'_>,
					title: &str,
					tab_focused: bool,
					parent_focused: bool,
				) -> BoundingBox {
					let bg_color = if tab_focused {
						ctx.config.editor_colors.bg
					} else {
						ctx.config.ui_colors.unfocused_bg
					};

					let text_color = if tab_focused && parent_focused {
						ctx.config.ui_colors.text
					} else {
						ctx.config.ui_colors.unfocused_text
					};

					let bounding_box = ctx
						.bounding_box
						.added_top(5)
						.added_left(5)
						.subbed_right(5);

					ctx.draw_solid_rect(bounding_box, bg_color);

					let text_bounding_box = bounding_box
						.added_y(bounding_box.h / 2)
						.added_left(15)
						.added_right(14);

					ctx.draw_text(Section {
						text: title,
						color: text_color.to_rgba(),
						font_id: ctx.text_renderer.ui_font(),
						scale: FontScale::uniform(14.0),
						layout: Layout::default()
							.v_align(VerticalAlign::Center),
						..text_bounding_box.to_section_bounds()
					});

					let icon_margin = bounding_box.h / 4;

					let icon_bounding_box = bounding_box
						.added_left(bounding_box.w - bounding_box.h)
						.added_top(icon_margin)
						.subbed_bottom(icon_margin)
						.added_left(icon_margin)
						.subbed_right(icon_margin);

					ctx.draw_icon_rect(icon_bounding_box, IconType::Close);

					ctx.bounding_box
				}

				let tabs_bounding_box = tabs_bg_bounding_box.added_left(10);

				let n: u32 = panes.len().try_into().unwrap();

				let tab_width = u32::min(175, ctx.bounding_box.w / n);

				for (i, pane) in panes.iter_mut().enumerate() {
					let i: u32 = i.try_into().unwrap();
					let tab_bounding_box = tabs_bounding_box
						.added_left(tab_width * i)
						.with_w(tab_width);

					draw_tab(
						ctx.with_bounding_box(tab_bounding_box),
						pane.title(),
						i == *focused,
						is_focused,
					);
				}

				panes[*focused as usize].draw(
					ctx.with_bounding_box(
						ctx.bounding_box.added_top(tabs_bg_bounding_box.h),
					),
					is_focused,
				);
			}
			Panes::Single(pane) => {
				pane.draw(ctx.reborrow(), ());
			}
		}
		ctx.bounding_box
	}
}

impl Drawable for Pane {
	type UserData = ();

	fn draw(&mut self, mut ctx: DrawingContext<'_>, _: ()) -> BoundingBox {
		ctx.draw_solid_rect(ctx.bounding_box, ctx.config.editor_colors.bg);
		ctx.bounding_box
	}
}

fn pixel_to_clip(x: u32, y: u32, w: u32, h: u32) -> Point {
	Point::new(
		(x as f32 / (w as f32) - 0.5) * 2.0,
		(y as f32 / (h as f32) - 0.5) * 2.0,
	)
}
