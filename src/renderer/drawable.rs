use super::{
	color::Color, text::TextRenderer, ColorVertex, TextureVertex, VertexIndex,
};

use crate::{
	config::Config,
	geometry::{
		bounding_box_ext::BoundingBoxExt,
		ext::{
			ScreenPixelPointExt, ScreenPixelRectExt, ScreenPixelSizeExt,
			TexPixelRectExt,
		},
		ScreenPixelPoint, ScreenPixelRect, ScreenPixelSize, TexNormPoint,
		TexNormRect, TexPixelRect,
	},
	icons::{IconType, Icons},
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
	bounding_box: ScreenPixelRect,
	window: &'a Window,
	config: &'a Config,
	icons: &'a Icons,
	color_vertex_buffer: &'a mut Vec<ColorVertex>,
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
		color_vertex_buffer: &'a mut Vec<ColorVertex>,
		color_index_buffer: &'a mut Vec<VertexIndex>,
		texture_vertex_buffer: &'a mut Vec<TextureVertex>,
		texture_index_buffer: &'a mut Vec<VertexIndex>,
		text_renderer: &'a mut TextRenderer,
	) -> DrawingContext<'a> {
		DrawingContext {
			bounding_box: ScreenPixelRect::new(
				ScreenPixelPoint::zero(),
				ScreenPixelSize::of_window(window),
			),
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
		bounding_box: ScreenPixelRect,
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

	fn draw_solid_rect(&mut self, bounding_box: ScreenPixelRect, color: Color) {
		let window_size = self
			.window
			.inner_size()
			.to_logical(self.window.scale_factor());

		let bounding_box = bounding_box.to_norm(window_size);

		let start_idx: u16 = self
			.color_vertex_buffer
			.len()
			.try_into()
			.expect("More than u16::MAX vertices");
		self.color_vertex_buffer.extend_from_slice(&[
			ColorVertex::new(bounding_box.top_left(), color),
			ColorVertex::new(bounding_box.top_right(), color),
			ColorVertex::new(bounding_box.bottom_right(), color),
			ColorVertex::new(bounding_box.bottom_left(), color),
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

	fn draw_icon_rect(
		&mut self,
		bounding_box: ScreenPixelRect,
		icon: IconType,
	) {
		let window_size = self
			.window
			.inner_size()
			.to_logical(self.window.scale_factor());
		let start_idx: u16 = self
			.texture_vertex_buffer
			.len()
			.try_into()
			.expect("More than u16::MAX vertices");

		let bounding_box = bounding_box.to_norm(window_size);

		let icon_rect = self.icons.get_icon_descriptor(icon);

		let atlas_size = self.icons.texture_atlas_size();

		let icon_rect = icon_rect.to_norm(atlas_size);

		self.texture_vertex_buffer.extend_from_slice(&[
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
	) -> ScreenPixelRect;
}

impl Drawable for Interface {
	type UserData = ();

	fn draw(&mut self, mut ctx: DrawingContext<'_>, _: ()) -> ScreenPixelRect {
		let mut tree_bounding_box = ctx.bounding_box.with_w(225);
		tree_bounding_box = self
			.tree_pane
			.draw(ctx.with_bounding_box(tree_bounding_box), ());
		let mut panes_bounding_box =
			ctx.bounding_box.deflate_left(tree_bounding_box.size.width);
		panes_bounding_box = self
			.panes
			.draw(ctx.with_bounding_box(panes_bounding_box), true);

		tree_bounding_box.added_w(panes_bounding_box.size.width)
	}
}

impl Drawable for TreePane {
	type UserData = ();

	fn draw(&mut self, mut ctx: DrawingContext<'_>, _: ()) -> ScreenPixelRect {
		ctx.draw_solid_rect(
			ctx.bounding_box,
			ctx.config.ui_colors.secondary_bg,
		);

		let mut current_bounding_box = ctx.bounding_box.deflate_top(8);

		let statuses_title_bounding_box =
			current_bounding_box.deflate_left(10).with_h(20);

		let statuses_title = Section {
			text: "Open REPLs",
			color: ctx.config.ui_colors.accent.to_rgba(),
			font_id: ctx.text_renderer.ui_font_medium(),
			scale: FontScale::uniform(20.0),
			..statuses_title_bounding_box.to_section_bounds()
		};
		ctx.draw_text(statuses_title);

		current_bounding_box = current_bounding_box
			.deflate_top(statuses_title_bounding_box.size.height)
			.deflate_top(10);

		let statuses_bounding_box = self
			.pane_statuses
			.draw(ctx.with_bounding_box(current_bounding_box), ());

		current_bounding_box = current_bounding_box
			.deflate_top(statuses_bounding_box.size.height)
			.deflate_top(40);

		let evaluators_title_bounding_box =
			current_bounding_box.with_h(20).deflate_left(10);

		let evaluators_title = Section {
			text: "Available REPLs",
			color: ctx.config.ui_colors.accent.to_rgba(),
			font_id: ctx.text_renderer.ui_font_medium(),
			scale: FontScale::uniform(20.0),
			..evaluators_title_bounding_box.to_section_bounds()
		};
		ctx.draw_text(evaluators_title);

		current_bounding_box = current_bounding_box
			.deflate_top(evaluators_title_bounding_box.size.height)
			.deflate_top(10);

		self.evaluators
			.draw(ctx.with_bounding_box(current_bounding_box), ());
		ctx.bounding_box
	}
}

impl Drawable for PaneStatuses {
	type UserData = ();

	fn draw(&mut self, mut ctx: DrawingContext<'_>, _: ()) -> ScreenPixelRect {
		let mut current_bounding_box = ctx.bounding_box;
		let mut drawn_bounding_box = ctx.bounding_box.with_h(0);

		for (i, pane_status) in self.pane_statuses.iter_mut().enumerate() {
			let i: u32 = i.try_into().unwrap();
			let mut inner_bounding_box = current_bounding_box.with_h(24);

			inner_bounding_box = pane_status.draw(
				ctx.with_bounding_box(inner_bounding_box),
				i == self.focused,
			);

			if drawn_bounding_box.bottom() + inner_bounding_box.size.height
				>= ctx.bounding_box.bottom()
			{
				break;
			}

			current_bounding_box = current_bounding_box
				.deflate_top(inner_bounding_box.size.height);
			drawn_bounding_box =
				drawn_bounding_box.with_bottom(inner_bounding_box.bottom());
		}

		ctx.bounding_box.with_h(drawn_bounding_box.size.height)
	}
}

impl Drawable for PaneStatus {
	type UserData = bool;

	fn draw(
		&mut self,
		mut ctx: DrawingContext<'_>,
		is_focused: bool,
	) -> ScreenPixelRect {
		if is_focused {
			ctx.draw_solid_rect(
				ctx.bounding_box,
				ctx.config.ui_colors.focused_bg,
			);
		}
		let text_bounding_box = ctx
			.bounding_box
			.deflate_left(20)
			.added_y(ctx.bounding_box.size.height / 2)
			.deflate_right(ctx.bounding_box.size.height);
		ctx.draw_text(Section {
			text: self.name.as_str(),
			color: ctx.config.ui_colors.text.to_rgba(),
			font_id: ctx.text_renderer.ui_font(),
			layout: Layout::default().v_align(VerticalAlign::Center),
			..text_bounding_box.to_section_bounds()
		});

		let icon_margin = u32::max(2, ctx.bounding_box.size.height / 5);
		let icon_bounding_box = ctx
			.bounding_box
			.deflate_left(
				ctx.bounding_box.size.width - ctx.bounding_box.size.height,
			)
			.with_w(ctx.bounding_box.size.height)
			.deflate(icon_margin);

		ctx.draw_icon_rect(icon_bounding_box, IconType::Close);

		ctx.bounding_box
	}
}

impl Drawable for Evaluators {
	type UserData = ();

	fn draw(&mut self, mut ctx: DrawingContext<'_>, _: ()) -> ScreenPixelRect {
		let mut current_bounding_box = ctx.bounding_box;
		let mut drawn_bounding_box = ctx.bounding_box.with_h(0);

		for evaluator in &mut self.evaluators {
			let mut inner_bounding_box = current_bounding_box.with_h(75);

			inner_bounding_box =
				evaluator.draw(ctx.with_bounding_box(inner_bounding_box), ());

			if drawn_bounding_box.bottom() + inner_bounding_box.size.height
				>= ctx.bounding_box.bottom()
			{
				break;
			}

			current_bounding_box = current_bounding_box
				.deflate_top(inner_bounding_box.size.height);
			drawn_bounding_box =
				drawn_bounding_box.with_bottom(inner_bounding_box.bottom());
		}
		ctx.bounding_box.with_h(drawn_bounding_box.size.height)
	}
}

impl Drawable for Evaluator {
	type UserData = ();

	fn draw(&mut self, mut ctx: DrawingContext<'_>, _: ()) -> ScreenPixelRect {
		ctx.draw_solid_rect(
			ctx.bounding_box
				.deflate_left(13)
				.deflate_right(13)
				.deflate_top(7)
				.deflate_bottom(7),
			ctx.config.ui_colors.bg,
		);

		ctx.draw_text(Section {
			text: self.name.as_str(),
			color: ctx.config.ui_colors.text.to_rgba(),
			font_id: ctx.text_renderer.ui_font_medium(),
			layout: Layout::default().v_align(VerticalAlign::Center),
			..ctx
				.bounding_box
				.deflate_left(22)
				.added_y(ctx.bounding_box.size.height / 2)
				.with_h(ctx.bounding_box.size.height / 3)
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
	) -> ScreenPixelRect {
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
					let total_width = ctx.bounding_box.size.width - 2 * (n - 1);
					let total_width = total_width as f64;
					let inner_pane_width = total_width / (n as f64);
					let inner_pane_width = inner_pane_width.ceil() as u32;
					ctx.bounding_box.with_w(inner_pane_width)
				};

				for (i, pane) in panes.iter_mut().enumerate() {
					let i: u32 = i.try_into().unwrap();
					pane.draw(
						ctx.with_bounding_box(
							inner_bounding_box.added_x(
								(inner_bounding_box.size.width + 2) * i,
							),
						),
						is_focused && i == *focused,
					);
					ctx.draw_solid_rect(
						inner_bounding_box
							.deflate_left(inner_bounding_box.size.width)
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
					ctx.bounding_box.with_h(ctx.bounding_box.size.height / n);

				for (i, pane) in panes.iter_mut().enumerate() {
					let i: u32 = i.try_into().unwrap();
					pane.draw(
						ctx.with_bounding_box(
							inner_bounding_box
								.added_y(inner_bounding_box.size.height * i),
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
				) -> ScreenPixelRect {
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
						.deflate_top(5)
						.deflate_left(5)
						.deflate_right(5);

					ctx.draw_solid_rect(bounding_box, bg_color);

					let icon_margin = bounding_box.size.height / 4;

					let text_bounding_box = bounding_box
						.added_y(bounding_box.size.height / 2)
						.deflate_left(15)
						.deflate_right(icon_margin);

					ctx.draw_text(Section {
						text: title,
						color: text_color.to_rgba(),
						font_id: ctx.text_renderer.ui_font(),
						scale: FontScale::uniform(14.0),
						layout: Layout::default()
							.v_align(VerticalAlign::Center),
						..text_bounding_box.to_section_bounds()
					});

					let icon_bounding_box = bounding_box
						.deflate_left(
							bounding_box.size.width - bounding_box.size.height,
						)
						.deflate(icon_margin);

					ctx.draw_icon_rect(icon_bounding_box, IconType::Close);

					ctx.bounding_box
				}

				let tabs_bounding_box = tabs_bg_bounding_box.deflate_left(10);

				let n: u32 = panes.len().try_into().unwrap();

				let tab_width = u32::min(175, ctx.bounding_box.size.width / n);

				for (i, pane) in panes.iter_mut().enumerate() {
					let i: u32 = i.try_into().unwrap();
					let tab_bounding_box = tabs_bounding_box
						.deflate_left(tab_width * i)
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
						ctx.bounding_box
							.deflate_top(tabs_bg_bounding_box.size.height),
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

	fn draw(&mut self, mut ctx: DrawingContext<'_>, _: ()) -> ScreenPixelRect {
		ctx.draw_solid_rect(ctx.bounding_box, ctx.config.editor_colors.bg);
		ctx.bounding_box
	}
}
