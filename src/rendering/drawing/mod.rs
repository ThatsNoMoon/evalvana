mod ctx;

pub use ctx::{DrawingBuffers, DrawingContext, DrawingId, DrawingManager};

mod tessellation;

use super::color::Color;

use crate::{
	config::EditorColors,
	geometry::{
		bounding_box_ext::BoundingBoxExt, ScreenPixelLength, ScreenPixelRect,
	},
	icons::IconType,
	interface::{
		Evaluator, Evaluators, Interface, Pane, PaneList, PaneStatus,
		PaneStatuses, Panes, TreePane,
	},
	repl::evaluation::{
		CompoundResult, Evaluation, Expression, PlainResult,
		Result as EvaluationResult,
	},
};

use std::convert::TryInto;

use wgpu_glyph::{Layout, Section, VerticalAlign};

pub trait Drawable {
	type UserData;
	fn draw(
		&mut self,
		ctx: &mut DrawingContext<'_>,
		bounds: ScreenPixelRect,
		data: Self::UserData,
	) -> ScreenPixelRect;
}

trait DrawableChild {
	type UserData;
	fn draw(
		&mut self,
		id: &DrawingId,
		ctx: &mut DrawingContext<'_>,
		bounds: ScreenPixelRect,
		data: Self::UserData,
	) -> ScreenPixelRect;
}

impl Drawable for Interface {
	type UserData = ();

	fn draw(
		&mut self,
		ctx: &mut DrawingContext<'_>,
		bounds: ScreenPixelRect,
		_: (),
	) -> ScreenPixelRect {
		if let Some(bounds) = self.drawn_bounds {
			return bounds;
		}

		ctx.clear(&self.drawing_id);

		let mut tree_bounds = bounds.with_w(225);
		tree_bounds = self.tree_pane.draw(ctx, tree_bounds, ());
		let mut panes_bounds = bounds.deflate_left(tree_bounds.size.width);
		panes_bounds =
			self.panes.draw(&self.drawing_id, ctx, panes_bounds, true);

		let drawn_bounds = tree_bounds.added_w(panes_bounds.size.width);

		self.drawn_bounds = Some(drawn_bounds);

		drawn_bounds
	}
}

impl Drawable for TreePane {
	type UserData = ();

	fn draw(
		&mut self,
		ctx: &mut DrawingContext<'_>,
		bounds: ScreenPixelRect,
		_: (),
	) -> ScreenPixelRect {
		if let Some(bounds) = self.drawn_bounds {
			return bounds;
		}

		ctx.clear(&self.drawing_id);

		ctx.draw_solid_rect(
			&self.drawing_id,
			bounds,
			ctx.config.ui_colors.secondary_bg,
		);

		let mut current_bounds = bounds.deflate_top(8);

		let statuses_title_bounds = current_bounds.deflate_left(10).with_h(20);

		let title_scale = {
			let mut scale = ctx.config.font_settings.ui_font_scale;
			scale.x *= 1.25;
			scale.y *= 1.25;
			scale
		};

		let statuses_title = Section {
			text: "Open REPLs",
			color: ctx.config.ui_colors.accent.to_rgba(),
			font_id: ctx.text_renderer.ui_font_medium(),
			scale: title_scale,
			layout: Layout::default_single_line(),
			..statuses_title_bounds.to_section_bounds()
		};
		ctx.draw_text(&self.drawing_id, statuses_title);

		current_bounds = current_bounds
			.deflate_top(statuses_title_bounds.size.height)
			.deflate_top(10);

		let statuses_bounds =
			self.pane_statuses
				.draw(&self.drawing_id, ctx, current_bounds, ());

		current_bounds = current_bounds
			.deflate_top(statuses_bounds.size.height)
			.deflate_top(40);

		let evaluators_title_bounds =
			current_bounds.with_h(20).deflate_left(10);

		let evaluators_title = Section {
			text: "Available REPLs",
			color: ctx.config.ui_colors.accent.to_rgba(),
			font_id: ctx.text_renderer.ui_font_medium(),
			scale: title_scale,
			layout: Layout::default_single_line(),
			..evaluators_title_bounds.to_section_bounds()
		};
		ctx.draw_text(&self.drawing_id, evaluators_title);

		current_bounds = current_bounds
			.deflate_top(evaluators_title_bounds.size.height)
			.deflate_top(10);

		self.evaluators
			.draw(&self.drawing_id, ctx, current_bounds, ());

		self.drawn_bounds = Some(bounds);

		bounds
	}
}

impl DrawableChild for PaneStatuses {
	type UserData = ();

	fn draw(
		&mut self,
		id: &DrawingId,
		ctx: &mut DrawingContext<'_>,
		mut bounds: ScreenPixelRect,
		_: (),
	) -> ScreenPixelRect {
		let mut current_bounds = bounds;
		let mut drawn_bounds = bounds.with_h(0);

		for (i, pane_status) in self.pane_statuses.iter_mut().enumerate() {
			let i: u32 = i.try_into().unwrap();
			let mut inner_bounds = current_bounds.with_h(24);

			inner_bounds =
				pane_status.draw(id, ctx, inner_bounds, i == self.focused);

			if drawn_bounds.bottom() + inner_bounds.size.height
				>= bounds.bottom()
			{
				break;
			}

			current_bounds =
				current_bounds.deflate_top(inner_bounds.size.height);
			drawn_bounds = drawn_bounds.with_bottom(inner_bounds.bottom());
		}

		bounds.size.height = drawn_bounds.size.height;

		self.drawn_bounds = Some(bounds);

		bounds
	}
}

impl DrawableChild for PaneStatus {
	type UserData = bool;

	fn draw(
		&mut self,
		id: &DrawingId,
		ctx: &mut DrawingContext<'_>,
		bounds: ScreenPixelRect,
		is_focused: bool,
	) -> ScreenPixelRect {
		if is_focused {
			ctx.draw_solid_rect(id, bounds, ctx.config.ui_colors.focused_bg);
		} else if self.hovered {
			ctx.draw_solid_rect(id, bounds, ctx.config.ui_colors.hovered_bg);
		}
		let text_bounds = bounds
			.deflate_left(20)
			.added_y(bounds.size.height / 2)
			.deflate_right(bounds.size.height);
		ctx.draw_text(
			id,
			Section {
				text: self.name.as_str(),
				color: ctx.config.ui_colors.text.to_rgba(),
				font_id: ctx.text_renderer.ui_font(),
				scale: ctx.config.font_settings.ui_font_scale,
				layout: Layout::default_single_line()
					.v_align(VerticalAlign::Center),
				..text_bounds.to_section_bounds()
			},
		);

		let icon_margin = u32::max(2, bounds.size.height / 5);
		let icon_bounds = bounds
			.deflate_left(bounds.size.width - bounds.size.height)
			.with_w(bounds.size.height)
			.deflate(icon_margin, icon_margin);

		ctx.draw_icon_rect(id, icon_bounds, IconType::Close);

		self.drawn_bounds = Some(bounds);

		bounds
	}
}

impl DrawableChild for Evaluators {
	type UserData = ();

	fn draw(
		&mut self,
		id: &DrawingId,
		ctx: &mut DrawingContext<'_>,
		mut bounds: ScreenPixelRect,
		_: (),
	) -> ScreenPixelRect {
		let mut current_bounds = bounds;
		let mut drawn_bounds = bounds.with_h(0);

		for evaluator in &mut self.evaluators {
			let mut inner_bounds = current_bounds.with_h(75);

			inner_bounds = evaluator.draw(id, ctx, inner_bounds, ());

			if drawn_bounds.bottom() + inner_bounds.size.height
				>= bounds.bottom()
			{
				break;
			}

			current_bounds =
				current_bounds.deflate_top(inner_bounds.size.height);
			drawn_bounds = drawn_bounds.with_bottom(inner_bounds.bottom());
		}

		bounds.size.height = drawn_bounds.size.height;

		self.drawn_bounds = Some(bounds);

		bounds
	}
}

impl DrawableChild for Evaluator {
	type UserData = ();

	fn draw(
		&mut self,
		id: &DrawingId,
		ctx: &mut DrawingContext<'_>,
		bounds: ScreenPixelRect,
		_: (),
	) -> ScreenPixelRect {
		{
			let bg = if self.hovered {
				ctx.config.ui_colors.hovered_bg
			} else {
				ctx.config.ui_colors.bg
			};

			ctx.draw_solid_rect(
				id,
				bounds
					.deflate_left(13)
					.deflate_right(13)
					.deflate_top(7)
					.deflate_bottom(7),
				bg,
			);
		}

		ctx.draw_text(
			id,
			Section {
				text: self.name.as_str(),
				color: ctx.config.ui_colors.text.to_rgba(),
				font_id: ctx.text_renderer.ui_font_medium(),
				layout: Layout::default_wrap().v_align(VerticalAlign::Center),
				..bounds
					.deflate_left(22)
					.added_y(bounds.size.height / 2)
					.with_h(bounds.size.height / 3)
					.to_section_bounds()
			},
		);

		self.drawn_bounds = Some(bounds);

		bounds
	}
}

impl DrawableChild for Panes {
	type UserData = bool;

	fn draw(
		&mut self,
		id: &DrawingId,
		ctx: &mut DrawingContext<'_>,
		bounds: ScreenPixelRect,
		is_focused: bool,
	) -> ScreenPixelRect {
		match self {
			Panes::VerticalSplit(PaneList { panes, focused }) => {
				if panes.is_empty() {
					ctx.draw_solid_rect(id, bounds, ctx.config.ui_colors.bg);
					return bounds;
				}

				let n: u32 = panes.len().try_into().unwrap();
				let inner_bounds = {
					let total_width = bounds.size.width - 2 * (n - 1);
					let total_width = total_width as f64;
					let inner_pane_width = total_width / (n as f64);
					let inner_pane_width = inner_pane_width.ceil() as u32;
					bounds.with_w(inner_pane_width)
				};

				for (i, pane) in panes.iter_mut().enumerate() {
					let i: u32 = i.try_into().unwrap();
					pane.draw(
						id,
						ctx,
						inner_bounds.added_x((inner_bounds.size.width + 2) * i),
						is_focused && i == *focused,
					);
					ctx.draw_solid_rect(
						id,
						inner_bounds
							.deflate_left(inner_bounds.size.width)
							.with_w(2),
						ctx.config.ui_colors.borders,
					);
				}
			}
			Panes::HorizontalSplit(PaneList { panes, focused }) => {
				if panes.is_empty() {
					ctx.draw_solid_rect(id, bounds, ctx.config.ui_colors.bg);
					return bounds;
				}

				let n: u32 = panes.len().try_into().unwrap();

				let inner_bounds = bounds.with_h(bounds.size.height / n);

				for (i, pane) in panes.iter_mut().enumerate() {
					let i: u32 = i.try_into().unwrap();
					pane.draw(
						id,
						ctx,
						inner_bounds.added_y(inner_bounds.size.height * i),
						is_focused && i == *focused,
					);
				}
			}
			Panes::Tabbed(PaneList { panes, focused }) => {
				if panes.is_empty() {
					ctx.draw_solid_rect(
						id,
						bounds,
						ctx.config.ui_colors.borders,
					);
					return bounds;
				}

				let tabs_bg_bounds = bounds.with_h(40);

				ctx.draw_solid_rect(
					id,
					tabs_bg_bounds,
					ctx.config.ui_colors.borders,
				);

				fn draw_tab(
					id: &DrawingId,
					ctx: &mut DrawingContext<'_>,
					bounds: ScreenPixelRect,
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

					let bounding_box =
						bounds.deflate_top(8).deflate_left(4).deflate_right(4);

					ctx.draw_solid_rect(id, bounding_box, bg_color);

					let icon_margin = bounding_box.size.height / 4;

					let text_bounds = bounding_box
						.added_y(bounding_box.size.height / 2)
						.deflate_left(15)
						.deflate_right(icon_margin);

					let title_scale = {
						let mut scale = ctx.config.font_settings.ui_font_scale;
						scale.x *= 0.875;
						scale.y *= 0.875;
						scale
					};

					ctx.draw_text(
						id,
						Section {
							text: title,
							color: text_color.to_rgba(),
							font_id: ctx.text_renderer.ui_font(),
							scale: title_scale,
							layout: Layout::default_single_line()
								.v_align(VerticalAlign::Center),
							..text_bounds.to_section_bounds()
						},
					);

					let icon_bounds = bounding_box
						.deflate_left(
							bounding_box.size.width - bounding_box.size.height,
						)
						.deflate(icon_margin, icon_margin);

					ctx.draw_icon_rect(id, icon_bounds, IconType::Close);

					bounds
				}

				let tabs_bounds = tabs_bg_bounds.deflate_left(4);

				let n: u32 = panes.len().try_into().unwrap();

				let tab_width = u32::min(175, bounds.size.width / n);

				for (i, pane) in panes.iter_mut().enumerate() {
					let i: u32 = i.try_into().unwrap();
					let tab_bounds = tabs_bounds
						.deflate_left(tab_width * i)
						.with_w(tab_width);

					draw_tab(
						id,
						ctx,
						tab_bounds,
						pane.title(),
						i == *focused,
						is_focused,
					);
				}

				panes[*focused as usize].draw(
					id,
					ctx,
					bounds.deflate_top(tabs_bg_bounds.size.height),
					is_focused,
				);
			}
			Panes::Single(pane) => {
				pane.draw(ctx, bounds, ());
			}
		}
		bounds
	}
}

impl Drawable for Pane {
	type UserData = ();

	fn draw(
		&mut self,
		ctx: &mut DrawingContext<'_>,
		bounds: ScreenPixelRect,
		_: (),
	) -> ScreenPixelRect {
		if let Some(bounds) = self.drawn_bounds {
			return bounds;
		}

		ctx.clear(&self.drawing_id);

		ctx.draw_solid_rect(
			&self.drawing_id,
			bounds,
			ctx.config.editor_colors.bg,
		);
		let mut current_bounds = bounds;
		let mut drawn_bounds = bounds.with_h(0);

		for evaluation in &mut self.evaluations {
			let inner_bounds =
				evaluation.draw(&self.drawing_id, ctx, current_bounds, ());

			current_bounds =
				current_bounds.deflate_top(inner_bounds.size.height);
			drawn_bounds = drawn_bounds.with_bottom(inner_bounds.bottom());

			if drawn_bounds.bottom() + inner_bounds.size.height
				>= bounds.bottom()
			{
				break;
			}
		}

		let total_bounds = bounds.with_h(drawn_bounds.size.height);

		self.drawn_bounds = Some(total_bounds);

		total_bounds
	}
}

impl DrawableChild for Evaluation {
	type UserData = ();

	fn draw(
		&mut self,
		id: &DrawingId,
		ctx: &mut DrawingContext<'_>,
		bounds: ScreenPixelRect,
		_: (),
	) -> ScreenPixelRect {
		let margin = ctx.config.font_settings.editor_font_scale.y.ceil() as u32;

		let mut current_bounds = bounds.deflate(margin * 2, margin * 2);
		let mut drawn_bounds = current_bounds.with_h(10);

		for section in [
			&mut self.input as &mut dyn DrawableChild<UserData = ()>,
			&mut self.output as &mut dyn DrawableChild<UserData = ()>,
		]
		.iter_mut()
		{
			let inner_bounds = section.draw(id, ctx, current_bounds, ());

			current_bounds =
				current_bounds.deflate_top(inner_bounds.size.height);
			drawn_bounds = drawn_bounds.with_bottom(inner_bounds.bottom());

			if drawn_bounds.bottom() + inner_bounds.size.height
				>= bounds.bottom()
			{
				break;
			}
		}

		ctx.draw_stroke_rect(
			id,
			drawn_bounds.inflate(margin, margin),
			ScreenPixelLength::new(1),
			ctx.config.ui_colors.borders,
		);
		bounds.with_h(drawn_bounds.size.height + margin * 4)
	}
}

impl DrawableChild for Expression {
	type UserData = ();

	fn draw(
		&mut self,
		id: &DrawingId,
		ctx: &mut DrawingContext<'_>,
		bounds: ScreenPixelRect,
		_: (),
	) -> ScreenPixelRect {
		let mut current_bounds = bounds.deflate(5, 5);
		let mut drawn_bounds = bounds.with_h(0);

		let font_scale = ctx.config.font_settings.editor_font_scale;

		let (line_height, digit_width, gutter_width) = {
			let line_digits =
				(self.input.lines().count() as f32).log10().ceil();

			let font =
				ctx.text_renderer.font_data(ctx.text_renderer.editor_font());
			let v_metrics = font.v_metrics(font_scale);
			let h_metrics = font
				.glyph('0')
				.scaled(ctx.config.font_settings.editor_font_scale)
				.h_metrics();

			(
				v_metrics.ascent + v_metrics.descent.abs() + v_metrics.line_gap,
				h_metrics.advance_width,
				h_metrics.advance_width * line_digits,
			)
		};

		let gutter_margin = digit_width.ceil() as u32 * 4;

		let line_height = line_height.ceil() as u32;
		let gutter_width = gutter_width.ceil() as u32 + gutter_margin;

		for (i, line) in self.input.lines().enumerate() {
			let inner_bounds = current_bounds.with_h(line_height);

			let gutter_bounds = inner_bounds.with_w(gutter_width);
			ctx.draw_text(
				id,
				Section {
					text: i.to_string().as_str(),
					scale: ctx.config.font_settings.editor_font_scale,
					color: ctx.config.editor_colors.gutter.to_rgba(),
					layout: Layout::default_single_line(),
					..gutter_bounds.to_section_bounds()
				},
			);

			let line_bounds = current_bounds.deflate_left(gutter_width);

			ctx.draw_text(
				id,
				Section {
					text: line,
					scale: ctx.config.font_settings.editor_font_scale,
					color: ctx.config.editor_colors.main.to_rgba(),
					layout: Layout::default_single_line(),
					..line_bounds.to_section_bounds()
				},
			);

			current_bounds =
				current_bounds.deflate_top(inner_bounds.size.height);
			drawn_bounds = drawn_bounds.with_bottom(inner_bounds.bottom());

			if drawn_bounds.bottom() + inner_bounds.size.height
				>= bounds.bottom()
			{
				break;
			}
		}
		bounds.with_h(drawn_bounds.size.height)
	}
}

impl DrawableChild for EvaluationResult {
	type UserData = ();

	fn draw(
		&mut self,
		id: &DrawingId,
		ctx: &mut DrawingContext<'_>,
		bounds: ScreenPixelRect,
		_: (),
	) -> ScreenPixelRect {
		let mut inner_bounds = bounds.deflate_top(
			ctx.config.font_settings.editor_font_scale.y.ceil() as u32,
		);
		ctx.draw_solid_rect(
			id,
			inner_bounds.with_h(1),
			ctx.config.editor_colors.gutter,
		);
		inner_bounds = match self {
			EvaluationResult::Empty => bounds.with_h(0),
			EvaluationResult::Success(plain_result) => {
				let color = ctx.config.editor_colors.success;
				plain_result.draw(id, ctx, inner_bounds, color)
			}
			EvaluationResult::Error(plain_result) => {
				let color = ctx.config.editor_colors.errors;
				plain_result.draw(id, ctx, inner_bounds, color)
			}
			EvaluationResult::Warning(plain_result) => {
				let color = ctx.config.editor_colors.warnings;
				plain_result.draw(id, ctx, inner_bounds, color)
			}
			EvaluationResult::Compound(compound_result) => {
				compound_result.draw(id, ctx, inner_bounds, ())
			}
		};
		inner_bounds.with_top(bounds.top())
	}
}

impl DrawableChild for PlainResult {
	type UserData = Color;

	fn draw(
		&mut self,
		id: &DrawingId,
		ctx: &mut DrawingContext<'_>,
		bounds: ScreenPixelRect,
		color: Color,
	) -> ScreenPixelRect {
		let line_height = {
			let font =
				ctx.text_renderer.font_data(ctx.text_renderer.editor_font());
			let v_metrics =
				font.v_metrics(ctx.config.font_settings.editor_font_scale);

			v_metrics.ascent + v_metrics.descent.abs() + v_metrics.line_gap
		};

		let n_lines = self.text.lines().count();
		let inner_bounds = bounds
			.deflate_top(
				ctx.config.font_settings.editor_font_scale.y.ceil() as u32
			)
			.with_h((n_lines as f32 * line_height).ceil() as u32);

		ctx.draw_text(
			id,
			Section {
				text: self.text.as_str(),
				color: color.to_rgba(),
				layout: Layout::default_wrap(),
				..inner_bounds.to_section_bounds()
			},
		);

		bounds.with_bottom(inner_bounds.bottom())
	}
}

impl DrawableChild for CompoundResult {
	type UserData = ();

	fn draw(
		&mut self,
		id: &DrawingId,
		ctx: &mut DrawingContext<'_>,
		bounds: ScreenPixelRect,
		_: (),
	) -> ScreenPixelRect {
		let mut current_bounds = bounds.deflate(5, 5);
		let mut drawn_bounds = bounds.with_h(0);

		let EditorColors {
			success,
			warnings,
			errors,
			..
		} = ctx.config.editor_colors;

		for (plain_result, color) in self
			.success
			.iter_mut()
			.map(|r| (r, success))
			.chain(self.warnings.iter_mut().map(|r| (r, warnings)))
			.chain(self.errors.iter_mut().map(|r| (r, errors)))
		{
			let inner_bounds =
				plain_result.draw(id, ctx, current_bounds, color);

			current_bounds =
				current_bounds.deflate_top(inner_bounds.size.height);
			drawn_bounds = drawn_bounds.with_bottom(inner_bounds.bottom());

			if drawn_bounds.bottom() + inner_bounds.size.height
				>= bounds.bottom()
			{
				break;
			}
		}

		bounds.with_h(drawn_bounds.size.height)
	}
}
