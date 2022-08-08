//! Display fields that can be filled with text.
//!
//! A [`TextInput`] has some local [`State`].

pub mod cursor;
mod editor;
mod rope_ext;
pub mod style;

use std::{borrow::Cow, ops::ControlFlow};

pub use cursor::Cursor;
use editor::Editor;
use iced_graphics::{alignment, Color, Vector};
use iced_native::{
	event::{self, Event},
	keyboard, layout,
	mouse::{self, click},
	renderer,
	text::{self, Text},
	touch, Clipboard, Element, Layout, Length, Padding, Point, Rectangle,
	Shell, Size, Widget,
};
use ordered_float::NotNan;
use rope_ext::RopeExt;
pub use ropey::Rope;
use ropey::RopeSlice;
use style::StyleSheet;
use unicode_segmentation::UnicodeSegmentation;

/// A field that can be filled with text.
///
/// # Example
/// ```
/// # use iced_native::renderer::Null;
/// # use iced_native::widget::text_input;
/// #
/// # pub type TextInput<'a, Message> = iced_native::widget::TextInput<'a, Message, Null>;
/// #[derive(Debug, Clone)]
/// enum Message {
///     TextInputChanged(String),
/// }
///
/// let mut state = text_input::State::new();
/// let value = "Some text";
///
/// let input = TextInput::new(
///     &mut state,
///     "This is the placeholder...",
///     value,
///     Message::TextInputChanged,
/// )
/// .padding(10);
/// ```
/// ![Text input drawn by `iced_wgpu`](https://github.com/iced-rs/iced/blob/7760618fb112074bc40b148944521f312152012a/docs/images/text_input.png?raw=true)
#[allow(missing_debug_implementations)]
pub struct TextInput<'a, Message, Renderer: text::Renderer> {
	state: &'a mut State,
	placeholder: String,
	font: Renderer::Font,
	width: Length,
	height: Length,
	padding: Padding,
	size: Option<u16>,
	tab_width: u8,
	on_change: Box<dyn Fn(String) -> Message + 'a>,
	on_submit: Option<Message>,
	style_sheet: Box<dyn StyleSheet + 'a>,
}

impl<'a, Message, Renderer> TextInput<'a, Message, Renderer>
where
	Message: Clone,
	Renderer: text::Renderer,
{
	/// Creates a new [`TextInput`].
	///
	/// It expects:
	/// - some [`State`]
	/// - a placeholder
	/// - the current value
	/// - a function that produces a message when the [`TextInput`] changes
	pub fn new<F>(state: &'a mut State, placeholder: &str, on_change: F) -> Self
	where
		F: 'a + Fn(String) -> Message,
	{
		TextInput {
			state,
			placeholder: String::from(placeholder),
			font: Default::default(),
			width: Length::Fill,
			height: Length::Fill,
			padding: Padding::ZERO,
			size: None,
			tab_width: 4,
			on_change: Box::new(on_change),
			on_submit: None,
			style_sheet: Default::default(),
		}
	}

	/// Sets the [`Font`] of the [`Text`].
	///
	/// [`Font`]: crate::widget::text::Renderer::Font
	/// [`Text`]: crate::widget::Text
	pub fn font(mut self, font: Renderer::Font) -> Self {
		self.font = font;
		self
	}
	/// Sets the width of the [`TextInput`].
	pub fn width(mut self, width: Length) -> Self {
		self.width = width;
		self
	}
	/// Sets the width of the [`TextInput`].
	pub fn height(mut self, height: Length) -> Self {
		self.height = height;
		self
	}

	/// Sets the [`Padding`] of the [`TextInput`].
	pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
		self.padding = padding.into();
		self
	}

	/// Sets the text size of the [`TextInput`].
	pub fn size(mut self, size: u16) -> Self {
		self.size = Some(size);
		self
	}

	/// Set the tab width of the [`TextInput`].
	pub fn tab_width(mut self, tab_width: u8) -> Self {
		self.tab_width = tab_width;
		self
	}

	/// Sets the message that should be produced when the [`TextInput`] is
	/// focused and the enter key is pressed.
	pub fn on_submit(mut self, message: Message) -> Self {
		self.on_submit = Some(message);
		self
	}

	/// Sets the style of the [`TextInput`].
	pub fn style(
		mut self,
		style_sheet: impl Into<Box<dyn StyleSheet + 'a>>,
	) -> Self {
		self.style_sheet = style_sheet.into();
		self
	}

	/// Returns the current [`State`] of the [`TextInput`].
	pub fn state(&self) -> &State {
		self.state
	}

	/// Draws the [`TextInput`] with the given [`Renderer`], overriding its
	/// [`Value`] if provided.
	pub fn draw(
		&self,
		renderer: &mut Renderer,
		layout: Layout<'_>,
		cursor_position: Point,
	) {
		draw(
			renderer,
			layout,
			cursor_position,
			self.state,
			&self.placeholder,
			self.size,
			self.tab_width,
			&self.font,
			self.style_sheet.as_ref(),
		)
	}
}

/// Computes the layout of a [`TextInput`].
pub fn layout<Renderer>(
	renderer: &Renderer,
	limits: &layout::Limits,
	width: Length,
	height: Length,
	value: &Rope,
	padding: Padding,
	size: Option<u16>,
) -> layout::Node
where
	Renderer: text::Renderer,
{
	let text_size = size.unwrap_or_else(|| renderer.default_size());

	let line_count = value.len_lines() + 1;

	let text_height = text_size as usize * line_count;

	let limits = limits.pad(padding).width(width).height(height);

	let mut text =
		layout::Node::new(limits.resolve(Size::new(0.0, text_height as f32)));
	text.move_to(Point::new(padding.left.into(), padding.top.into()));

	layout::Node::with_children(text.size().pad(padding), vec![text])
}

/// Processes an [`Event`] and updates the [`State`] of a [`TextInput`]
/// accordingly.
#[allow(clippy::too_many_arguments)]
pub fn update<'a, Message, Renderer>(
	event: Event,
	layout: Layout<'_>,
	cursor_position: Point,
	renderer: &Renderer,
	clipboard: &mut dyn Clipboard,
	shell: &mut Shell<'_, Message>,
	size: Option<u16>,
	tab_width: u8,
	font: &Renderer::Font,
	on_change: &dyn Fn(String) -> Message,
	on_submit: &Option<Message>,
	state: impl FnOnce() -> &'a mut State,
) -> event::Status
where
	Message: Clone,
	Renderer: text::Renderer,
{
	let state = state();
	let size = size.unwrap_or_else(|| renderer.default_size());
	let text_bounds = layout.children().next().unwrap().bounds();

	state.new_size(size);

	match event {
		Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
		| Event::Touch(touch::Event::FingerPressed { .. }) => {
			let is_clicked = layout.bounds().contains(cursor_position);

			state.is_focused = is_clicked;

			if is_clicked {
				let offset = cursor_position - text_bounds.position();
				let click =
					mouse::Click::new(cursor_position, state.last_click);

				match click.kind() {
					click::Kind::Single => {
						let position = if offset != Vector::new(0.0, 0.0) {
							index_at_point(
								renderer,
								font.clone(),
								size,
								tab_width,
								state,
								Point::ORIGIN + offset,
							)
						} else {
							None
						};

						state.cursor.move_to_byte(position.unwrap_or(0));
						state.is_dragging = true;
					}
					click::Kind::Double => {
						let position = index_at_point(
							renderer,
							font.clone(),
							size,
							tab_width,
							state,
							Point::ORIGIN + offset,
						)
						.unwrap_or(0);

						state.cursor.select_range(
							state.value.previous_start_of_word(position),
							state.value.next_end_of_word(position),
						);

						state.is_dragging = false;
					}
					click::Kind::Triple => {
						state.cursor.select_all(&state.value);
						state.is_dragging = false;
					}
				}

				state.last_click = Some(click);

				return event::Status::Captured;
			}
		}
		Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
		| Event::Touch(touch::Event::FingerLifted { .. })
		| Event::Touch(touch::Event::FingerLost { .. }) => {
			state.is_dragging = false;
		}
		Event::Mouse(mouse::Event::CursorMoved { position })
		| Event::Touch(touch::Event::FingerMoved { position, .. }) => {
			if state.is_dragging {
				let offset = position - text_bounds.position();

				let position = index_at_point(
					renderer,
					font.clone(),
					size,
					tab_width,
					state,
					Point::ORIGIN + offset,
				)
				.unwrap_or(0);

				state
					.cursor
					.select_range(state.cursor.start(&state.value), position);

				return event::Status::Captured;
			}
		}
		Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
			let delta = match delta {
				mouse::ScrollDelta::Lines { x, y } => {
					let size = f32::from(size);
					Vector::new(x * size, y * size * -1.0)
				}
				mouse::ScrollDelta::Pixels { x, y } => Vector::new(x, y),
			};

			if delta.y.abs() > 0.1 {
				state.scroll.y = (state.scroll.y + delta.y)
					.max(0.0)
					.min(state.value.len_lines() as f32 * f32::from(size));
			}

			if delta.x.abs() > 0.1 {
				let max = (max_line_length(
					&state.value,
					renderer,
					font.clone(),
					size,
					tab_width,
				) - text_bounds.width)
					.max(0.0);
				state.scroll.x = (state.scroll.x + delta.x).max(0.0).min(max);
			}
		}
		Event::Keyboard(keyboard::Event::CharacterReceived(c)) => {
			if state.is_focused
				&& state.is_pasting.is_none()
				&& !state.keyboard_modifiers.command()
				&& (!c.is_control() || c == '\n' || c == '\r' || c == '\t')
			{
				let mut editor =
					Editor::new(&mut state.value, &mut state.cursor);

				editor.insert(c);

				if c == '\r' {
					editor.insert('\n');
				}

				let message = (on_change)(editor.contents());
				shell.publish(message);

				state.recalculate_scroll_offset(
					renderer,
					text_bounds.size(),
					font.clone(),
					size,
					tab_width,
				);

				return event::Status::Captured;
			}
		}
		Event::Keyboard(keyboard::Event::KeyPressed { key_code, .. }) => {
			if state.is_focused {
				let modifiers = state.keyboard_modifiers;

				match key_code {
					keyboard::KeyCode::Enter
					| keyboard::KeyCode::NumpadEnter
						if !state.keyboard_modifiers.control() =>
					{
						if let Some(on_submit) = on_submit.clone() {
							shell.publish(on_submit);
						}
					}
					keyboard::KeyCode::Backspace => {
						if platform::is_jump_modifier_pressed(modifiers)
							&& state.cursor.selection(&state.value).is_none()
						{
							state.cursor.select_left_by_words(&state.value);
						}

						let mut editor =
							Editor::new(&mut state.value, &mut state.cursor);
						editor.backspace();

						let message = (on_change)(editor.contents());
						shell.publish(message);

						state.recalculate_scroll_offset(
							renderer,
							text_bounds.size(),
							font.clone(),
							size,
							tab_width,
						);
					}
					keyboard::KeyCode::Delete => {
						if platform::is_jump_modifier_pressed(modifiers)
							&& state.cursor.selection(&state.value).is_none()
						{
							state.cursor.select_right_by_words(&state.value);
						}

						let mut editor =
							Editor::new(&mut state.value, &mut state.cursor);
						editor.delete();

						let message = (on_change)(editor.contents());
						shell.publish(message);

						state.recalculate_scroll_offset(
							renderer,
							text_bounds.size(),
							font.clone(),
							size,
							tab_width,
						);
					}
					keyboard::KeyCode::Left => {
						if platform::is_jump_modifier_pressed(modifiers) {
							if modifiers.shift() {
								state.cursor.select_left_by_words(&state.value);
							} else {
								state.cursor.move_left_by_words(&state.value);
							}
						} else if modifiers.shift() {
							state.cursor.select_left(&state.value)
						} else {
							state.cursor.move_left(&state.value);
						}

						state.recalculate_scroll_offset(
							renderer,
							text_bounds.size(),
							font.clone(),
							size,
							tab_width,
						);
					}
					keyboard::KeyCode::Right => {
						if platform::is_jump_modifier_pressed(modifiers) {
							if modifiers.shift() {
								state
									.cursor
									.select_right_by_words(&state.value);
							} else {
								state.cursor.move_right_by_words(&state.value);
							}
						} else if modifiers.shift() {
							state.cursor.select_right(&state.value)
						} else {
							state.cursor.move_right(&state.value);
						}

						state.recalculate_scroll_offset(
							renderer,
							text_bounds.size(),
							font.clone(),
							size,
							tab_width,
						);
					}
					keyboard::KeyCode::Up => {
						if modifiers.shift() {
							state.cursor.select_up(
								&state.value,
								renderer,
								font.clone(),
								tab_width,
							)
						} else {
							state.cursor.move_up(
								&state.value,
								renderer,
								font.clone(),
								tab_width,
							);
						}

						state.recalculate_scroll_offset(
							renderer,
							text_bounds.size(),
							font.clone(),
							size,
							tab_width,
						);
					}
					keyboard::KeyCode::Down => {
						if modifiers.shift() {
							state.cursor.select_down(
								&state.value,
								renderer,
								font.clone(),
								tab_width,
							)
						} else {
							state.cursor.move_down(
								&state.value,
								renderer,
								font.clone(),
								tab_width,
							);
						}

						state.recalculate_scroll_offset(
							renderer,
							text_bounds.size(),
							font.clone(),
							size,
							tab_width,
						);
					}
					keyboard::KeyCode::Home => {
						if platform::is_jump_modifier_pressed(modifiers) {
							if modifiers.shift() {
								state.cursor.select_range(
									state.cursor.start(&state.value),
									0,
								);
							} else {
								state.cursor.move_to_byte(0);
							}

							state.scroll = Vector::new(0.0, 0.0);
						} else {
							if modifiers.shift() {
								state.cursor.select_left_by_line(&state.value);
							} else {
								state.cursor.move_left_by_line(&state.value);
							}

							state.recalculate_scroll_offset(
								renderer,
								text_bounds.size(),
								font.clone(),
								size,
								tab_width,
							);
						}
					}
					keyboard::KeyCode::End => {
						if platform::is_jump_modifier_pressed(modifiers) {
							if modifiers.shift() {
								state.cursor.select_range(
									state.cursor.start(&state.value),
									state.value.len_bytes(),
								);
							} else {
								state
									.cursor
									.move_to_byte(state.value.len_bytes());
							}
						} else if modifiers.shift() {
							state.cursor.select_right_by_line(&state.value);
						} else {
							state.cursor.move_right_by_line(&state.value);
						}

						state.recalculate_scroll_offset(
							renderer,
							text_bounds.size(),
							font.clone(),
							size,
							tab_width,
						);
					}
					keyboard::KeyCode::C
						if state.keyboard_modifiers.command() =>
					{
						match state.cursor.selection(&state.value) {
							Some((start, end)) => {
								clipboard.write(
									state
										.value
										.byte_slice(start..end)
										.to_string(),
								);
							}
							None => {}
						}
					}
					keyboard::KeyCode::X
						if state.keyboard_modifiers.command() =>
					{
						match state.cursor.selection(&state.value) {
							Some((start, end)) => {
								clipboard.write(
									state
										.value
										.byte_slice(start..end)
										.to_string(),
								);

								let mut editor = Editor::new(
									&mut state.value,
									&mut state.cursor,
								);
								editor.delete();

								let message = (on_change)(editor.contents());
								shell.publish(message);

								state.recalculate_scroll_offset(
									renderer,
									text_bounds.size(),
									font.clone(),
									size,
									tab_width,
								);
							}
							None => {}
						}
					}
					keyboard::KeyCode::V => {
						if state.keyboard_modifiers.command() {
							let content: String = match state.is_pasting.take()
							{
								Some(content) => content,
								None => clipboard
									.read()
									.unwrap_or_default()
									.chars()
									.filter(|&c| {
										!c.is_control()
											|| c == '\n' || c == '\r' || c == '\t'
									})
									.collect(),
							};

							let mut editor = Editor::new(
								&mut state.value,
								&mut state.cursor,
							);

							editor.paste(&content);

							let message = (on_change)(editor.contents());
							shell.publish(message);

							state.is_pasting = Some(content);

							state.recalculate_scroll_offset(
								renderer,
								text_bounds.size(),
								font.clone(),
								size,
								tab_width,
							);
						} else {
							state.is_pasting = None;
						}
					}
					keyboard::KeyCode::A
						if state.keyboard_modifiers.command() =>
					{
						state.cursor.select_all(&state.value);

						state.recalculate_scroll_offset(
							renderer,
							text_bounds.size(),
							font.clone(),
							size,
							tab_width,
						);
					}
					keyboard::KeyCode::Escape => {
						state.is_dragging = false;
						state.is_pasting = None;

						state.keyboard_modifiers =
							keyboard::Modifiers::default();

						state.recalculate_scroll_offset(
							renderer,
							text_bounds.size(),
							font.clone(),
							size,
							tab_width,
						);
					}
					keyboard::KeyCode::Tab => {
						return event::Status::Ignored;
					}
					_ => {}
				}

				return event::Status::Captured;
			}
		}
		Event::Keyboard(keyboard::Event::KeyReleased { key_code, .. }) => {
			if state.is_focused {
				match key_code {
					keyboard::KeyCode::V => {
						state.is_pasting = None;
					}
					keyboard::KeyCode::Tab
					| keyboard::KeyCode::Up
					| keyboard::KeyCode::Down => {
						return event::Status::Ignored;
					}
					_ => {}
				}

				return event::Status::Captured;
			}
		}
		Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
			if state.is_focused {
				state.keyboard_modifiers = modifiers;
			}
		}
		_ => {}
	}

	event::Status::Ignored
}

/// Draws the [`TextInput`] with the given [`Renderer`], overriding its
/// [`Value`] if provided.
#[allow(clippy::too_many_arguments)]
pub fn draw<Renderer>(
	renderer: &mut Renderer,
	layout: Layout<'_>,
	pointer_position: Point,
	state: &State,
	placeholder: &str,
	size: Option<u16>,
	tab_width: u8,
	font: &Renderer::Font,
	style_sheet: &dyn StyleSheet,
) where
	Renderer: text::Renderer,
{
	let bounds = layout.bounds();
	let text_bounds = layout.children().next().unwrap().bounds();
	let value = &state.value;

	let is_mouse_over = bounds.contains(pointer_position);

	let style = if state.is_focused() {
		style_sheet.focused()
	} else if is_mouse_over {
		style_sheet.hovered()
	} else {
		style_sheet.active()
	};

	renderer.fill_quad(
		renderer::Quad {
			bounds,
			border_radius: style.border_radius,
			border_width: style.border_width,
			border_color: style.border_color,
		},
		style.background,
	);

	let size = size.unwrap_or_else(|| renderer.default_size());

	let (selections, cursor) = if state.is_focused() {
		match state.cursor.state(value) {
			cursor::State::Index(position) => {
				let point = offset_of_index(
					position,
					value,
					renderer,
					font.clone(),
					size,
					tab_width,
				);

				(vec![], Some(point))
			}
			cursor::State::Selection { start, end } => {
				let left = start.min(end);
				let right = end.max(start);

				let (left_point, right_point) = {
					let left_y =
						value.byte_to_line(left) as f32 * f32::from(size);
					let right_y = left_y
						+ (value.byte_slice(left..right).len_lines() - 1)
							as f32 * f32::from(size);

					let left_x = offset_x_of_index(
						left,
						value,
						renderer,
						font.clone(),
						Some(size),
						tab_width,
					);
					let right_x = offset_x_of_index(
						right,
						value,
						renderer,
						font.clone(),
						Some(size),
						tab_width,
					);

					(Point::new(left_x, left_y), Point::new(right_x, right_y))
				};

				let selection_quads = if left_point.y == right_point.y {
					vec![(
						renderer::Quad {
							bounds: Rectangle {
								x: text_bounds.x + left_point.x,
								y: text_bounds.y + left_point.y,
								width: right_point.x - left_point.x,
								height: f32::from(size),
							},
							border_radius: 0.0,
							border_width: 0.0,
							border_color: Color::TRANSPARENT,
						},
						style_sheet.selection_color(),
					)]
				} else {
					let mut selections = vec![];

					let color = style_sheet.selection_color();

					let quad = |start_point: Point, width| renderer::Quad {
						bounds: Rectangle {
							x: text_bounds.x + start_point.x,
							y: text_bounds.y + start_point.y,
							width,
							height: f32::from(size),
						},
						border_radius: 0.0,
						border_width: 0.0,
						border_color: Color::TRANSPARENT,
					};

					let mut line_start = left;

					let mut line_index = value.byte_to_line(line_start);

					let mut start_point = left_point;

					loop {
						let line_end = value.line_to_byte(line_index + 1);

						let mut width = width_of_range(
							line_start,
							line_end.min(right),
							value,
							renderer,
							font.clone(),
							Some(size),
							tab_width,
						);

						if value.byte(line_end.min(right) - 1) == b'\n' {
							width += f32::from(size) / 2.0;
						}

						selections.push((quad(start_point, width), color));

						if line_end >= right {
							break;
						}

						line_start = line_end;
						start_point =
							Point::new(0.0, start_point.y + f32::from(size));

						line_index += 1;
					}

					selections
				};

				(
					selection_quads,
					if end < start {
						Some(left_point)
					} else {
						Some(right_point)
					},
				)
			}
		}
	} else {
		(vec![], None)
	};

	let cursor = cursor
		.map(|point| {
			point + (text_bounds.position() - Point::ORIGIN) - state.scroll
		})
		.filter(|&point| {
			let bottom = point + Vector::new(0.0, f32::from(size));
			text_bounds.contains(point) || text_bounds.contains(bottom)
		})
		.map(|point| {
			let y = f32::max(point.y - 1.0, text_bounds.y);

			let height = f32::min(
				f32::from(size) + 2.0,
				text_bounds.y + text_bounds.height - y,
			);

			(
				renderer::Quad {
					bounds: Rectangle {
						x: point.x - 1.0,
						y,
						width: 2.0,
						height,
					},
					border_radius: 0.0,
					border_width: 0.0,
					border_color: Color::TRANSPARENT,
				},
				style_sheet.cursor_color(),
			)
		});

	let render = |renderer: &mut Renderer| {
		for (selection, color) in selections {
			renderer.fill_quad(selection, color);
		}
		let color = if value.len_bytes() == 0 {
			style_sheet.placeholder_color()
		} else {
			style_sheet.value_color()
		};

		let size = f32::from(size);

		if value.len_bytes() == 0 {
			renderer.fill_text(Text {
				content: placeholder,
				color,
				font: font.clone(),
				bounds: Rectangle {
					width: f32::INFINITY,
					height: f32::INFINITY,
					..text_bounds
				},
				size,
				horizontal_alignment: alignment::Horizontal::Left,
				vertical_alignment: alignment::Vertical::Top,
			});
			return;
		}

		let first_line = (state.scroll.y / size).floor() as usize;

		let line_count = (text_bounds.height / size).ceil() as usize;

		let lines = value.byte_slice(
			value.line_to_byte(first_line)
				..=value
					.line_to_byte(
						(first_line + line_count).min(value.len_lines()),
					)
					.min(value.len_bytes() - 1),
		);

		let text = lines.display(tab_width);

		for (i, mut line) in text.enumerate() {
			if i == line_count && line == "" {
				line = " ".into();
			}
			renderer.fill_text(Text {
				content: &line,
				color,
				font: font.clone(),
				bounds: Rectangle {
					x: text_bounds.x,
					y: text_bounds.y + (i + first_line) as f32 * size,
					width: f32::INFINITY,
					height: size,
				},
				size,
				horizontal_alignment: alignment::Horizontal::Left,
				vertical_alignment: alignment::Vertical::Top,
			});
		}
	};

	renderer.with_layer(text_bounds, |renderer| {
		renderer.with_translation(state.scroll * -1.0, render);
	});

	if let Some((cursor, color)) = cursor {
		renderer.fill_quad(cursor, color);
	}
}

/// Computes the current [`mouse::Interaction`] of the [`TextInput`].
pub fn mouse_interaction(
	layout: Layout<'_>,
	cursor_position: Point,
) -> mouse::Interaction {
	if layout.bounds().contains(cursor_position) {
		mouse::Interaction::Text
	} else {
		mouse::Interaction::default()
	}
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
	for TextInput<'a, Message, Renderer>
where
	Message: Clone,
	Renderer: text::Renderer,
{
	fn width(&self) -> Length {
		self.width
	}

	fn height(&self) -> Length {
		self.height
	}

	fn layout(
		&self,
		renderer: &Renderer,
		limits: &layout::Limits,
	) -> layout::Node {
		layout(
			renderer,
			limits,
			self.width,
			self.height,
			&self.state.value,
			self.padding,
			self.size,
		)
	}

	fn on_event(
		&mut self,
		event: Event,
		layout: Layout<'_>,
		cursor_position: Point,
		renderer: &Renderer,
		clipboard: &mut dyn Clipboard,
		shell: &mut Shell<'_, Message>,
	) -> event::Status {
		update(
			event,
			layout,
			cursor_position,
			renderer,
			clipboard,
			shell,
			self.size,
			self.tab_width,
			&self.font,
			self.on_change.as_ref(),
			&self.on_submit,
			|| &mut self.state,
		)
	}

	fn mouse_interaction(
		&self,
		layout: Layout<'_>,
		cursor_position: Point,
		_viewport: &Rectangle,
		_renderer: &Renderer,
	) -> mouse::Interaction {
		mouse_interaction(layout, cursor_position)
	}

	fn draw(
		&self,
		renderer: &mut Renderer,
		_style: &renderer::Style,
		layout: Layout<'_>,
		cursor_position: Point,
		_viewport: &Rectangle,
	) {
		self.draw(renderer, layout, cursor_position)
	}
}

impl<'a, Message, Renderer> From<TextInput<'a, Message, Renderer>>
	for Element<'a, Message, Renderer>
where
	Message: 'a + Clone,
	Renderer: 'a + text::Renderer,
{
	fn from(
		text_input: TextInput<'a, Message, Renderer>,
	) -> Element<'a, Message, Renderer> {
		Element::new(text_input)
	}
}

/// The state of a [`TextInput`].
#[derive(Debug, Clone)]
pub struct State {
	value: Rope,
	is_focused: bool,
	is_dragging: bool,
	is_pasting: Option<String>,
	last_click: Option<mouse::Click>,
	cursor: Cursor,
	keyboard_modifiers: keyboard::Modifiers,
	scroll: Vector,
	last_size: u16,
}

impl Default for State {
	fn default() -> Self {
		Self {
			value: Rope::new(),
			is_focused: false,
			is_dragging: false,
			is_pasting: None,
			last_click: None,
			cursor: Cursor::default(),
			keyboard_modifiers: keyboard::Modifiers::default(),
			scroll: Vector::new(0.0, 0.0),
			last_size: 1,
		}
	}
}

impl State {
	/// Creates a new [`State`], representing a focused [`TextInput`].
	pub fn focused() -> Self {
		Self {
			is_focused: true,
			..Default::default()
		}
	}

	pub fn contents(&self) -> String {
		self.value.to_string()
	}

	/// Returns whether the [`TextInput`] is currently focused or not.
	fn is_focused(&self) -> bool {
		self.is_focused
	}

	fn recalculate_scroll_offset<Renderer: text::Renderer>(
		&mut self,
		renderer: &Renderer,
		bounds_size: Size<f32>,
		font: Renderer::Font,
		size: u16,
		tab_width: u8,
	) {
		let cursor_index = self.cursor.end(&self.value);
		let cursor = offset_of_index(
			cursor_index,
			&self.value,
			renderer,
			font,
			size,
			tab_width,
		);

		let x = if cursor.x < self.scroll.x {
			cursor.x
		} else if cursor.x > self.scroll.x + bounds_size.width {
			cursor.x - bounds_size.width
		} else {
			self.scroll.x
		};

		let y = if cursor.y < self.scroll.y {
			cursor.y
		} else if cursor.y + f32::from(size)
			> self.scroll.y + bounds_size.height
		{
			cursor.y + f32::from(size) - bounds_size.height
		} else {
			self.scroll.y
		};

		self.scroll = Vector::new(x, y);
	}

	fn new_size(&mut self, size: u16) {
		if size != self.last_size {
			let factor = f32::from(size) / f32::from(self.last_size);
			self.last_size = size;

			self.scroll = self.scroll * factor;
		}
	}
}

mod platform {
	use crate::keyboard;

	pub fn is_jump_modifier_pressed(modifiers: keyboard::Modifiers) -> bool {
		if cfg!(target_os = "macos") {
			modifiers.alt()
		} else {
			modifiers.control()
		}
	}
}

/// Computes the position of the text cursor at the given point of a
/// [`TextInput`].
fn index_at_point<Renderer>(
	renderer: &Renderer,
	font: Renderer::Font,
	size: u16,
	tab_width: u8,
	state: &State,
	mut point: Point,
) -> Option<usize>
where
	Renderer: text::Renderer,
{
	point = point + state.scroll;

	let line_num = (point.y / f32::from(size)).floor() as usize;

	let line_start = match state.value.try_line_to_byte(line_num) {
		Ok(i) if i < state.value.len_bytes() => i,
		_ => return Some(state.value.len_bytes()),
	};

	let line = state.value.line(line_num);

	let line_text = line
		.display(tab_width)
		.next()
		.expect("No line produced for hit test");

	if line_text.trim().is_empty() {
		return Some(line_start);
	}

	hit_byte_index(
		renderer,
		line,
		line_text.as_ref(),
		size,
		font,
		tab_width,
		point,
	)
	.map(|offset| line_start + offset)
}

fn hit_byte_index<'t, Renderer: text::Renderer>(
	renderer: &Renderer,
	line: RopeSlice<'_>,
	line_text: impl Into<Option<&'t str>>,
	size: u16,
	font: Renderer::Font,
	tab_width: u8,
	point: Point,
) -> Option<usize> {
	let line_text = line_text.into().map_or_else(
		|| {
			line.display(tab_width)
				.next()
				.expect("No line produced for hit test")
		},
		Into::into,
	);

	renderer
		.hit_test(&line_text, size.into(), font, Size::INFINITY, point, true)
		.map(text::Hit::cursor)
		.map(|index| {
			if index == line_text.len() {
				return line.len_bytes();
			}
			let byte_index = line_text
				.grapheme_indices(true)
				.nth(index)
				.expect("Hit test produced out of bounds grapheme index")
				.0;

			let tab_width = usize::from(tab_width);

			let result = line.bytes().try_fold((0, 0), |(i, num_tabs), b| {
				if i >= byte_index {
					let num_virtual_spaces =
						num_tabs * (tab_width.saturating_sub(1)) as usize;

					ControlFlow::Break(byte_index - num_virtual_spaces)
				} else if b == b'\t' {
					if i + tab_width > byte_index {
						let num_virtual_spaces =
							num_tabs * (tab_width.saturating_sub(1)) as usize;
						if (byte_index - i) <= tab_width / 2 {
							ControlFlow::Break(i - num_virtual_spaces)
						} else {
							ControlFlow::Break(i - num_virtual_spaces + 1)
						}
					} else {
						ControlFlow::Continue((i + tab_width, num_tabs + 1))
					}
				} else {
					ControlFlow::Continue((i + 1, num_tabs))
				}
			});

			match result {
				ControlFlow::Continue(_) => {
					panic!("Hit test produced invalid byte index")
				}
				ControlFlow::Break(i) => i,
			}
		})
}

fn offset_x_of_index<Renderer>(
	index: usize,
	value: &Rope,
	renderer: &Renderer,
	font: Renderer::Font,
	size: Option<u16>,
	tab_width: u8,
) -> f32
where
	Renderer: text::Renderer,
{
	let line_start = value.line_to_byte(value.byte_to_line(index));
	width_of_range(line_start, index, value, renderer, font, size, tab_width)
}

fn offset_y_of_index(index: usize, value: &Rope, size: u16) -> f32 {
	let lines_before = value.byte_to_line(index);
	lines_before as f32 * f32::from(size)
}

fn width_of_range<Renderer>(
	start: usize,
	end: usize,
	value: &Rope,
	renderer: &Renderer,
	font: Renderer::Font,
	size: Option<u16>,
	tab_width: u8,
) -> f32
where
	Renderer: text::Renderer,
{
	let size = size.unwrap_or_else(|| renderer.default_size());

	let space_width = renderer.measure_width(" ", size, font.clone());
	width_of_slice(
		value.byte_slice(start..end),
		renderer,
		font,
		size,
		tab_width,
		space_width,
	)
}

fn offset_of_index<Renderer>(
	index: usize,
	value: &Rope,
	renderer: &Renderer,
	font: Renderer::Font,
	size: u16,
	tab_width: u8,
) -> Point
where
	Renderer: text::Renderer,
{
	Point::new(
		offset_x_of_index(index, value, renderer, font, Some(size), tab_width),
		offset_y_of_index(index, value, size),
	)
}

fn max_line_length<Renderer>(
	value: &Rope,
	renderer: &Renderer,
	font: Renderer::Font,
	size: u16,
	tab_width: u8,
) -> f32
where
	Renderer: text::Renderer,
{
	let space_width = renderer.measure_width(" ", size, font.clone());

	value
		.lines()
		.map(|s| {
			NotNan::new(width_of_slice(
				s,
				renderer,
				font.clone(),
				size,
				tab_width,
				space_width,
			))
			.unwrap()
		})
		.max()
		.map(|x| x.into_inner())
		.unwrap_or(0.0)
}

fn width_of_slice<Renderer: text::Renderer>(
	slice: RopeSlice<'_>,
	renderer: &Renderer,
	font: Renderer::Font,
	size: u16,
	tab_width: u8,
	space_width: f32,
) -> f32 {
	let mut chunks = slice.chunks();

	let mut width = 0.0;

	let mut s = match chunks.next() {
		Some(s) => Cow::Borrowed(s),
		None => return width,
	};

	loop {
		let mut i = 0;
		let mut text_start = 0;
		let b = s.as_bytes();

		while i < b.len() {
			if b[i] == b'\t' {
				let text = &s[text_start..i];
				if !text.is_empty() {
					width += renderer.measure_width(text, size, font.clone());
				}

				let tab_start = i;
				i += 1;

				while i < b.len() && b[i] == b'\t' {
					i += 1;
				}

				width +=
					space_width * f32::from(tab_width) * (i - tab_start) as f32;
				text_start = i;
			} else {
				i += 1;
			}
		}

		s = if text_start == s.len() {
			match chunks.next() {
				Some(s) => Cow::Borrowed(s),
				None => {
					let text = &s[text_start..i];
					if !text.is_empty() {
						width += renderer.measure_width(text, size, font);
					}
					return width;
				}
			}
		} else {
			match chunks.next() {
				Some(next) => Cow::Owned(s[text_start..].to_owned() + next),
				None => {
					let text = &s[text_start..i];
					if !text.is_empty() {
						width += renderer.measure_width(text, size, font);
					}
					return width;
				}
			}
		};
	}
}

#[cfg(test)]
mod tests {
	use std::iter::repeat;

	use iced_graphics::Font;
	use iced_native::text::Renderer;
	use ropey::{Rope, RopeBuilder};

	use super::*;

	struct Mock;

	impl iced_native::Renderer for Mock {
		fn with_layer(&mut self, _: Rectangle, f: impl FnOnce(&mut Self)) {
			f(self);
		}

		fn with_translation(&mut self, _: Vector, f: impl FnOnce(&mut Self)) {
			f(self)
		}

		fn clear(&mut self) {}

		fn fill_quad(
			&mut self,
			_: renderer::Quad,
			_: impl Into<iced_graphics::Background>,
		) {
		}
	}

	impl Renderer for Mock {
		type Font = Font;

		const ICON_FONT: Self::Font = Font::Default;

		const CHECKMARK_ICON: char = '✅';

		const ARROW_DOWN_ICON: char = '⬇';

		fn default_size(&self) -> u16 {
			12
		}

		fn measure(
			&self,
			content: &str,
			size: u16,
			_: Self::Font,
			bounds: Size,
		) -> (f32, f32) {
			(
				((content.len() * size as usize) as f32).min(bounds.width),
				f32::from(size).min(bounds.height),
			)
		}

		fn hit_test(
			&self,
			_: &str,
			_: f32,
			_: Font,
			_: Size,
			_: Point,
			_: bool,
		) -> Option<text::Hit> {
			unimplemented!()
		}

		fn fill_text(&mut self, _: Text<'_, Self::Font>) {}
	}

	#[test]
	fn mock_text_renderer() {
		assert_eq!(Mock.measure_width(" ", 10, Font::default()), 10.0);
		assert_eq!(Mock.measure_width("hello", 10, Font::default()), 50.0);
	}

	#[test]
	fn width_of_slice_basic() {
		let rope = Rope::from_str("hello");
		assert_eq!(
			width_of_slice(rope.slice(..), &Mock, Font::default(), 10, 4, 10.0),
			50.0
		);
	}

	#[test]
	fn width_of_slice_long() {
		let iters = 10_000;
		let string = "this is a long string ";
		let rope = {
			let mut builder = RopeBuilder::new();
			for s in repeat(string).take(iters) {
				builder.append(s)
			}
			builder.finish()
		};

		{
			let mut chunks = rope.chunks();

			chunks.next();

			assert_ne!(chunks.next(), None);
		}

		let size = 10;
		assert_eq!(
			width_of_slice(
				rope.slice(..),
				&Mock,
				Font::default(),
				size,
				4,
				size.into()
			),
			(iters * string.len() * usize::from(size)) as f32
		);
	}

	#[test]
	fn width_of_slice_tabs() {
		let rope = Rope::from_str("\t\thello\tworld");
		let size = 10;
		let tab_width = 4;
		assert_eq!(
			width_of_slice(
				rope.slice(..),
				&Mock,
				Font::default(),
				size,
				tab_width,
				size.into()
			),
			(3 * u16::from(tab_width) * size + 10 * size) as f32
		);
	}
}
