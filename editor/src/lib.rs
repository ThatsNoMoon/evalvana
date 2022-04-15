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
	is_secure: bool,
	font: Renderer::Font,
	width: Length,
	height: Length,
	padding: Padding,
	size: Option<u16>,
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
			is_secure: false,
			font: Default::default(),
			width: Length::Fill,
			height: Length::Fill,
			padding: Padding::ZERO,
			size: None,
			on_change: Box::new(on_change),
			on_submit: None,
			style_sheet: Default::default(),
		}
	}

	/// Converts the [`TextInput`] into a secure password input.
	pub fn password(mut self) -> Self {
		self.is_secure = true;
		self
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
			&self.font,
			self.is_secure,
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
	font: &Renderer::Font,
	is_secure: bool,
	on_change: &dyn Fn(String) -> Message,
	on_submit: &Option<Message>,
	state: impl FnOnce() -> &'a mut State,
) -> event::Status
where
	Message: Clone,
	Renderer: text::Renderer,
{
	match event {
		Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
		| Event::Touch(touch::Event::FingerPressed { .. }) => {
			let state = state();
			let is_clicked = layout.bounds().contains(cursor_position);

			state.is_focused = is_clicked;

			if is_clicked {
				let text_layout = layout.children().next().unwrap();
				let offset = cursor_position - text_layout.bounds().position();

				let click =
					mouse::Click::new(cursor_position, state.last_click);

				match click.kind() {
					click::Kind::Single => {
						let position = if offset != Vector::new(0.0, 0.0) {
							let value = if is_secure {
								value.secure()
							} else {
								value.clone()
							};

							find_cursor_position(
								renderer,
								text_layout.bounds(),
								font.clone(),
								size,
								&value,
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
						if is_secure {
							state.cursor.select_all(value);
						} else {
							let position = find_cursor_position(
								renderer,
								text_layout.bounds(),
								font.clone(),
								size,
								value,
								state,
								Point::ORIGIN + offset,
							)
							.unwrap_or(0);

							state.cursor.select_range(
								value.previous_start_of_word(position),
								value.next_end_of_word(position),
							);
						}

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
			state().is_dragging = false;
		}
		Event::Mouse(mouse::Event::CursorMoved { position })
		| Event::Touch(touch::Event::FingerMoved { position, .. }) => {
			let state = state();

			if state.is_dragging {
				let text_layout = layout.children().next().unwrap();
				let offset = position - text_layout.bounds().position();

				let value = if is_secure {
					value.secure()
				} else {
					value.clone()
				};

				let position = find_cursor_position(
					renderer,
					text_layout.bounds(),
					font.clone(),
					size,
					&value,
					state,
					Point::ORIGIN + offset,
				)
				.unwrap_or(0);

				state
					.cursor
					.select_range(state.cursor.start(&value), position);

				return event::Status::Captured;
			}
		}
		Event::Keyboard(keyboard::Event::CharacterReceived(c)) => {
			let state = state();

			if state.is_focused
				&& state.is_pasting.is_none()
				&& !state.keyboard_modifiers.command()
				&& (!c.is_control() || c == '\n' || c == '\r')
			{
				let mut editor = Editor::new(value, &mut state.cursor);

				editor.insert(c);

				if c == '\r' {
					editor.insert('\n');
				}

				let message = (on_change)(editor.contents());
				shell.publish(message);

				return event::Status::Captured;
			}
		}
		Event::Keyboard(keyboard::Event::KeyPressed { key_code, .. }) => {
			let state = state();

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
							if is_secure {
								let cursor_pos = state.cursor.end(value);
								state.cursor.select_range(0, cursor_pos);
							} else {
								state.cursor.select_left_by_words(value);
							}
						}

						let mut editor = Editor::new(value, &mut state.cursor);
						editor.backspace();

						let message = (on_change)(editor.contents());
						shell.publish(message);
					}
					keyboard::KeyCode::Delete => {
						if platform::is_jump_modifier_pressed(modifiers)
							&& state.cursor.selection(value).is_none()
						{
							if is_secure {
								let cursor_pos = state.cursor.end(value);
								state
									.cursor
									.select_range(cursor_pos, value.len());
							} else {
								state.cursor.select_right_by_words(value);
							}
						}

						let mut editor = Editor::new(value, &mut state.cursor);
						editor.delete();

						let message = (on_change)(editor.contents());
						shell.publish(message);
					}
					keyboard::KeyCode::Left => {
						if platform::is_jump_modifier_pressed(modifiers)
							&& !is_secure
						{
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
					}
					keyboard::KeyCode::Right => {
						if platform::is_jump_modifier_pressed(modifiers)
							&& !is_secure
						{
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
					}
					keyboard::KeyCode::Up => {
						if modifiers.shift() {
							state.cursor.select_up(
								value,
								renderer,
								font.clone(),
							)
						} else {
							state.cursor.move_up(value, renderer, font.clone());
						}
					}
					keyboard::KeyCode::Down => {
						if modifiers.shift() {
							state.cursor.select_down(
								value,
								renderer,
								font.clone(),
							)
						} else {
							state.cursor.move_down(
								value,
								renderer,
								font.clone(),
							);
						}
					}
					keyboard::KeyCode::Home => {
						if platform::is_jump_modifier_pressed(modifiers)
							&& !is_secure
						{
							if modifiers.shift() {
								state
									.cursor
									.select_range(state.cursor.start(value), 0);
							} else {
								state.cursor.move_to(0);
							}
						} else if modifiers.shift() {
							state.cursor.select_left_by_line(value);
						} else {
							state.cursor.move_left_by_line(value);
						}
					}
					keyboard::KeyCode::End => {
						if platform::is_jump_modifier_pressed(modifiers)
							&& !is_secure
						{
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
					}
					keyboard::KeyCode::C
						if state.keyboard_modifiers.command() =>
					{
						match state.cursor.selection(value) {
							Some((start, end)) => {
								clipboard.write(
									value.select(start, end).to_string(),
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
									value.select(start, end).to_string(),
								);
							}
							None => {}
						}

						let mut editor = Editor::new(value, &mut state.cursor);
						editor.delete();

						let message = (on_change)(editor.contents());
						shell.publish(message);
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
												|| c == '\n' || c == '\r'
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
						} else {
							state.is_pasting = None;
						}
					}
					keyboard::KeyCode::A
						if state.keyboard_modifiers.command() =>
					{
						state.cursor.select_all(value);
					}
					keyboard::KeyCode::Escape => {
						state.is_focused = false;
						state.is_dragging = false;
						state.is_pasting = None;

						state.keyboard_modifiers =
							keyboard::Modifiers::default();
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
			let state = state();

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
			let state = state();

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
	cursor_position: Point,
	state: &State,
	value: &Value,
	placeholder: &str,
	size: Option<u16>,
	font: &Renderer::Font,
	is_secure: bool,
	style_sheet: &dyn StyleSheet,
) where
	Renderer: text::Renderer,
{
	let secure_value = is_secure.then(|| value.secure());
	let value = secure_value.as_ref().unwrap_or(value);

	let bounds = layout.bounds();
	let text_bounds = layout.children().next().unwrap().bounds();

	let is_mouse_over = bounds.contains(cursor_position);

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

	let text = value.to_string();
	let size = size.unwrap_or_else(|| renderer.default_size());

	let (selection, cursor, offset) = if state.is_focused() {
		match state.cursor.state(value) {
			cursor::State::Index(position) => {
				let (point, offset) = measure_cursor_and_scroll_offset(
					renderer,
					text_bounds,
					value,
					size,
					position,
					font.clone(),
				);

				(None, Some(point), offset)
			}
			cursor::State::Selection { start, end } => {
				let left = start.min(end);
				let right = end.max(start);

				let (left_point, left_offset) =
					measure_cursor_and_scroll_offset(
						renderer,
						text_bounds,
						value,
						size,
						left,
						font.clone(),
					);

				let (right_point, right_offset) =
					measure_cursor_and_scroll_offset(
						renderer,
						text_bounds,
						value,
						size,
						right,
						font.clone(),
					);

				(
					Some((
						renderer::Quad {
							bounds: Rectangle {
								x: text_bounds.x + left_point.x,
								y: text_bounds.y + left_point.y,
								width: right_point.x - left_point.x,
								height: right_point.y - left_point.y
									+ f32::from(size),
							},
							border_radius: 0.0,
							border_width: 0.0,
							border_color: Color::TRANSPARENT,
						},
						style_sheet.selection_color(),
					)),
					if end < start {
						Some(left_point)
					} else {
						Some(right_point)
					},
					if end == right {
						right_offset
					} else {
						left_offset
					},
				)
			}
		}
	} else {
		(None, None, 0.0)
	};

	let cursor = cursor.map(|point| {
		(
			renderer::Quad {
				bounds: Rectangle {
					x: text_bounds.x + point.x - 1.0,
					y: text_bounds.y + point.y - 1.0,
					width: 2.0,
					height: f32::from(size) + 2.0,
				},
				border_radius: 0.0,
				border_width: 0.0,
				border_color: Color::TRANSPARENT,
			},
			style_sheet.cursor_color(),
		)
	});

	let text_width = renderer.measure_width(
		if text.is_empty() { placeholder } else { &text },
		size,
		font.clone(),
	);

	let render = |renderer: &mut Renderer| {
		if let Some((selection, color)) = selection {
			renderer.fill_quad(selection, color);
		}

		if let Some((cursor, color)) = cursor {
			renderer.fill_quad(cursor, color);
		}

		let color = if text.is_empty() {
			style_sheet.placeholder_color()
		} else {
			style_sheet.value_color()
		};

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
				..text_bounds
			},
			size: f32::from(size),
			horizontal_alignment: alignment::Horizontal::Left,
			vertical_alignment: alignment::Vertical::Top,
		});
	};

	if text_width > text_bounds.width {
		renderer.with_layer(text_bounds, |renderer| {
			renderer.with_translation(Vector::new(-offset, 0.0), render)
		});
	} else {
		render(renderer);
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
			&self.font,
			self.is_secure,
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
#[derive(Debug, Default, Clone)]
pub struct State {
	is_focused: bool,
	is_dragging: bool,
	is_pasting: Option<Value>,
	last_click: Option<mouse::Click>,
	cursor: Cursor,
	keyboard_modifiers: keyboard::Modifiers,
	// TODO: Add stateful horizontal scrolling offset
}

impl State {
	/// Creates a new [`State`], representing an unfocused [`TextInput`].
	pub fn new() -> Self {
		Self::default()
	}

	/// Creates a new [`State`], representing a focused [`TextInput`].
	pub fn focused() -> Self {
		Self {
			is_focused: true,
			is_dragging: false,
			is_pasting: None,
			last_click: None,
			cursor: Cursor::default(),
			keyboard_modifiers: keyboard::Modifiers::default(),
		}
	}

	/// Returns whether the [`TextInput`] is currently focused or not.
	pub fn is_focused(&self) -> bool {
		self.is_focused
	}

	/// Returns the [`Cursor`] of the [`TextInput`].
	pub fn cursor(&self) -> Cursor {
		self.cursor
	}

	/// Focuses the [`TextInput`].
	pub fn focus(&mut self) {
		self.is_focused = true;
	}

	/// Unfocuses the [`TextInput`].
	pub fn unfocus(&mut self) {
		self.is_focused = false;
	}

	/// Moves the [`Cursor`] of the [`TextInput`] to the front of the input text.
	pub fn move_cursor_to_front(&mut self) {
		self.cursor.move_to(0);
	}

	/// Moves the [`Cursor`] of the [`TextInput`] to the end of the input text.
	pub fn move_cursor_to_end(&mut self) {
		self.cursor.move_to(usize::MAX);
	}

	/// Moves the [`Cursor`] of the [`TextInput`] to an arbitrary location.
	pub fn move_cursor_to(&mut self, position: usize) {
		self.cursor.move_to(position);
	}

	/// Selects all the content of the [`TextInput`].
	pub fn select_all(&mut self) {
		self.cursor.select_range(0, usize::MAX);
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

fn offset<Renderer>(
	renderer: &Renderer,
	text_bounds: Rectangle,
	font: Renderer::Font,
	size: u16,
	value: &Value,
	state: &State,
) -> f32
where
	Renderer: text::Renderer,
{
	if state.is_focused() {
		let cursor = state.cursor();

		let focus_position = match cursor.state(value) {
			cursor::State::Index(i) => i,
			cursor::State::Selection { end, .. } => end,
		};

		let (_, offset) = measure_cursor_and_scroll_offset(
			renderer,
			text_bounds,
			value,
			size,
			focus_position,
			font,
		);

		offset
	} else {
		0.0
	}
}

fn measure_cursor_and_scroll_offset<Renderer>(
	renderer: &Renderer,
	text_bounds: Rectangle,
	value: &Value,
	size: u16,
	cursor_index: usize,
	font: Renderer::Font,
) -> (Point, f32)
where
	Renderer: text::Renderer,
{
	let lines_before_cursor = value.count_lines_before(cursor_index);

	let line_before_cursor = value
		.select(value.previous_start_of_line(cursor_index), cursor_index)
		.to_string();

	let text_value_width =
		renderer.measure_width(&line_before_cursor, size, font);

	let offset = ((text_value_width + 5.0) - text_bounds.width).max(0.0);

	let point =
		Point::new(text_value_width, lines_before_cursor as f32 * size as f32);

	(point, offset)
}

/// Computes the position of the text cursor at the given point of a
/// [`TextInput`].
fn find_cursor_position<Renderer>(
	renderer: &Renderer,
	text_bounds: Rectangle,
	font: Renderer::Font,
	size: Option<u16>,
	value: &Value,
	state: &State,
	mut point: Point,
) -> Option<usize>
where
	Renderer: text::Renderer,
{
	let size = size.unwrap_or_else(|| renderer.default_size());

	let offset =
		offset(renderer, text_bounds, font.clone(), size, value, state);

	point.x += offset;

	renderer
		.hit_test(
			&value.to_string(),
			size.into(),
			font,
			Size::INFINITY,
			point,
			true,
		)
		.map(text::Hit::cursor)
}
