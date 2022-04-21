//! Display fields that can be filled with text.
//!
//! A [`TextInput`] has some local [`State`].

pub mod cursor;
mod editor;
pub mod style;
mod value;

use std::borrow::Cow;

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
use style::StyleSheet;
pub use value::Value;

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
	value: Value,
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
	pub fn new<F>(
		state: &'a mut State,
		placeholder: &str,
		value: &str,
		on_change: F,
	) -> Self
	where
		F: 'a + Fn(String) -> Message,
	{
		TextInput {
			state,
			placeholder: String::from(placeholder),
			value: Value::new(value),
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
		value: Option<&Value>,
	) {
		draw(
			renderer,
			layout,
			cursor_position,
			self.state,
			value.unwrap_or(&self.value),
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
	value: &Value,
	padding: Padding,
	size: Option<u16>,
) -> layout::Node
where
	Renderer: text::Renderer,
{
	let text_size = size.unwrap_or_else(|| renderer.default_size());

	let line_count = value.count_lines() + 1;

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
	value: &mut Value,
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
								value,
								state,
								Point::ORIGIN + offset,
							)
						} else {
							None
						};

						state.cursor.move_to(position.unwrap_or(0));
						state.is_dragging = true;
					}
					click::Kind::Double => {
						let position = index_at_point(
							renderer,
							font.clone(),
							size,
							tab_width,
							value,
							state,
							Point::ORIGIN + offset,
						)
						.unwrap_or(0);

						state.cursor.select_range(
							value.previous_start_of_word(position),
							value.next_end_of_word(position),
						);

						state.is_dragging = false;
					}
					click::Kind::Triple => {
						state.cursor.select_all(value);
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
					value,
					state,
					Point::ORIGIN + offset,
				)
				.unwrap_or(0);

				state
					.cursor
					.select_range(state.cursor.start(value), position);

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
					.min(value.count_lines() as f32 * f32::from(size));
			}

			if delta.x.abs() > 0.1 {
				let max = (max_line_length(
					value,
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
				let mut editor = Editor::new(value, &mut state.cursor);

				editor.insert(c);

				if c == '\r' {
					editor.insert('\n');
				}

				let message = (on_change)(editor.contents());
				shell.publish(message);

				state.recalculate_scroll_offset(
					renderer,
					value,
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
							&& state.cursor.selection(value).is_none()
						{
							state.cursor.select_left_by_words(value);
						}

						let mut editor = Editor::new(value, &mut state.cursor);
						editor.backspace();

						let message = (on_change)(editor.contents());
						shell.publish(message);

						state.recalculate_scroll_offset(
							renderer,
							value,
							text_bounds.size(),
							font.clone(),
							size,
							tab_width,
						);
					}
					keyboard::KeyCode::Delete => {
						if platform::is_jump_modifier_pressed(modifiers)
							&& state.cursor.selection(value).is_none()
						{
							state.cursor.select_right_by_words(value);
						}

						let mut editor = Editor::new(value, &mut state.cursor);
						editor.delete();

						let message = (on_change)(editor.contents());
						shell.publish(message);

						state.recalculate_scroll_offset(
							renderer,
							value,
							text_bounds.size(),
							font.clone(),
							size,
							tab_width,
						);
					}
					keyboard::KeyCode::Left => {
						if platform::is_jump_modifier_pressed(modifiers) {
							if modifiers.shift() {
								state.cursor.select_left_by_words(value);
							} else {
								state.cursor.move_left_by_words(value);
							}
						} else if modifiers.shift() {
							state.cursor.select_left(value)
						} else {
							state.cursor.move_left(value);
						}

						state.recalculate_scroll_offset(
							renderer,
							value,
							text_bounds.size(),
							font.clone(),
							size,
							tab_width,
						);
					}
					keyboard::KeyCode::Right => {
						if platform::is_jump_modifier_pressed(modifiers) {
							if modifiers.shift() {
								state.cursor.select_right_by_words(value);
							} else {
								state.cursor.move_right_by_words(value);
							}
						} else if modifiers.shift() {
							state.cursor.select_right(value)
						} else {
							state.cursor.move_right(value);
						}

						state.recalculate_scroll_offset(
							renderer,
							value,
							text_bounds.size(),
							font.clone(),
							size,
							tab_width,
						);
					}
					keyboard::KeyCode::Up => {
						if modifiers.shift() {
							state.cursor.select_up(
								value,
								renderer,
								font.clone(),
								tab_width,
							)
						} else {
							state.cursor.move_up(
								value,
								renderer,
								font.clone(),
								tab_width,
							);
						}

						state.recalculate_scroll_offset(
							renderer,
							value,
							text_bounds.size(),
							font.clone(),
							size,
							tab_width,
						);
					}
					keyboard::KeyCode::Down => {
						if modifiers.shift() {
							state.cursor.select_down(
								value,
								renderer,
								font.clone(),
								tab_width,
							)
						} else {
							state.cursor.move_down(
								value,
								renderer,
								font.clone(),
								tab_width,
							);
						}

						state.recalculate_scroll_offset(
							renderer,
							value,
							text_bounds.size(),
							font.clone(),
							size,
							tab_width,
						);
					}
					keyboard::KeyCode::Home => {
						if platform::is_jump_modifier_pressed(modifiers) {
							if modifiers.shift() {
								state
									.cursor
									.select_range(state.cursor.start(value), 0);
							} else {
								state.cursor.move_to(0);
							}

							state.scroll = Vector::new(0.0, 0.0);
						} else {
							if modifiers.shift() {
								state.cursor.select_left_by_line(value);
							} else {
								state.cursor.move_left_by_line(value);
							}

							state.recalculate_scroll_offset(
								renderer,
								value,
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
									state.cursor.start(value),
									value.len(),
								);
							} else {
								state.cursor.move_to(value.len());
							}
						} else if modifiers.shift() {
							state.cursor.select_right_by_line(value);
						} else {
							state.cursor.move_right_by_line(value);
						}

						state.recalculate_scroll_offset(
							renderer,
							value,
							text_bounds.size(),
							font.clone(),
							size,
							tab_width,
						);
					}
					keyboard::KeyCode::C
						if state.keyboard_modifiers.command() =>
					{
						match state.cursor.selection(value) {
							Some((start, end)) => {
								clipboard.write(
									value.select(start, end).to_string(None),
								);
							}
							None => {}
						}
					}
					keyboard::KeyCode::X
						if state.keyboard_modifiers.command() =>
					{
						match state.cursor.selection(value) {
							Some((start, end)) => {
								clipboard.write(
									value.select(start, end).to_string(None),
								);

								let mut editor =
									Editor::new(value, &mut state.cursor);
								editor.delete();

								let message = (on_change)(editor.contents());
								shell.publish(message);

								state.recalculate_scroll_offset(
									renderer,
									value,
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
							let content = match state.is_pasting.take() {
								Some(content) => content,
								None => {
									let content: String = clipboard
										.read()
										.unwrap_or_default()
										.chars()
										.filter(|&c| {
											!c.is_control()
												|| c == '\n' || c == '\r' || c
												== '\t'
										})
										.collect();

									Value::new(&content)
								}
							};

							let mut editor =
								Editor::new(value, &mut state.cursor);

							editor.paste(content.clone());

							let message = (on_change)(editor.contents());
							shell.publish(message);

							state.is_pasting = Some(content);

							state.recalculate_scroll_offset(
								renderer,
								value,
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
						state.cursor.select_all(value);

						state.recalculate_scroll_offset(
							renderer,
							value,
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
							value,
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
	value: &Value,
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
						value.count_lines_before(left) as f32 * f32::from(size);
					let right_y = left_y
						+ value.count_lines_between(left, right) as f32
							* f32::from(size);

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

					let mut start_point = left_point;

					loop {
						let line_end =
							value.next_end_of_line(line_start).min(right);

						let width = if line_end == line_start {
							if line_end == right || line_start == left {
								0.0
							} else {
								f32::from(size) / 2.0
							}
						} else {
							width_of_range(
								line_start,
								line_end,
								value,
								renderer,
								font.clone(),
								Some(size),
								tab_width,
							)
						};

						selections.push((quad(start_point, width), color));

						line_start = line_end + 1;
						start_point =
							Point::new(0.0, start_point.y + f32::from(size));

						if line_end == right {
							break;
						}
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
		let color = if value.is_empty() {
			style_sheet.placeholder_color()
		} else {
			style_sheet.value_color()
		};

		let text = value.to_string(tab_width);

		let content: Cow<str> = match text.chars().next_back() {
			None => placeholder.into(),
			Some('\n') => {
				let mut t = text;
				t.push(' ');
				t.into()
			}
			Some(_) => text.into(),
		};

		renderer.fill_text(Text {
			content: content.as_ref(),
			color,
			font: font.clone(),
			bounds: Rectangle {
				width: f32::INFINITY,
				height: f32::INFINITY,
				..text_bounds
			},
			size: f32::from(size),
			horizontal_alignment: alignment::Horizontal::Left,
			vertical_alignment: alignment::Vertical::Top,
		});
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
			&self.value,
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
			&mut self.value,
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
		self.draw(renderer, layout, cursor_position, None)
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
	is_focused: bool,
	is_dragging: bool,
	is_pasting: Option<Value>,
	last_click: Option<mouse::Click>,
	cursor: Cursor,
	keyboard_modifiers: keyboard::Modifiers,
	scroll: Vector,
	last_size: u16,
}

impl Default for State {
	fn default() -> Self {
		Self {
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

	/// Returns whether the [`TextInput`] is currently focused or not.
	fn is_focused(&self) -> bool {
		self.is_focused
	}

	fn recalculate_scroll_offset<Renderer: text::Renderer>(
		&mut self,
		renderer: &Renderer,
		value: &Value,
		bounds_size: Size<f32>,
		font: Renderer::Font,
		size: u16,
		tab_width: u8,
	) {
		let cursor_index = self.cursor.end(value);
		let cursor = offset_of_index(
			cursor_index,
			value,
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
	value: &Value,
	state: &State,
	mut point: Point,
) -> Option<usize>
where
	Renderer: text::Renderer,
{
	point = point + state.scroll;

	let line_num = (point.y / f32::from(size)).floor() as usize;

	let line_start = match value.nth_line_start(line_num) {
		Some(i) => i,
		None => return Some(value.len()),
	};

	let line_end = value.next_end_of_line(line_start);

	let line = value.select(line_start, line_end).to_string(tab_width);

	if line.is_empty() {
		return Some(line_start);
	}

	renderer
		.hit_test(&line, size.into(), font, Size::INFINITY, point, true)
		.map(text::Hit::cursor)
		.map(|index| {
			let num_tabs = value.count_tabs_between(line_start, line_end);
			let num_virtual_spaces =
				num_tabs * (tab_width.saturating_sub(1)) as usize;
			index + line_start - num_virtual_spaces
		})
}

fn offset_x_of_index<Renderer>(
	index: usize,
	value: &Value,
	renderer: &Renderer,
	font: Renderer::Font,
	size: Option<u16>,
	tab_width: u8,
) -> f32
where
	Renderer: text::Renderer,
{
	let line_start = value.previous_start_of_line(index);
	width_of_range(line_start, index, value, renderer, font, size, tab_width)
}

fn offset_y_of_index(index: usize, value: &Value, size: u16) -> f32 {
	let lines_before = value.count_lines_before(index);
	lines_before as f32 * f32::from(size)
}

fn width_of_range<Renderer>(
	start: usize,
	end: usize,
	value: &Value,
	renderer: &Renderer,
	font: Renderer::Font,
	size: Option<u16>,
	tab_width: u8,
) -> f32
where
	Renderer: text::Renderer,
{
	let range = value.select(start, end);
	renderer.measure_width(
		&range.to_string(tab_width),
		size.unwrap_or_else(|| renderer.default_size()),
		font,
	)
}

fn offset_of_index<Renderer>(
	index: usize,
	value: &Value,
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
	value: &Value,
	renderer: &Renderer,
	font: Renderer::Font,
	size: u16,
	tab_width: u8,
) -> f32
where
	Renderer: text::Renderer,
{
	value
		.lines(tab_width)
		.map(|s| {
			NotNan::new(renderer.measure_width(&s, size, font.clone())).unwrap()
		})
		.max()
		.map(|x| x.into_inner())
		.unwrap_or(0.0)
}
