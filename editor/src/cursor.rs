//! Track the cursor of a text input.
use iced_graphics::Point;
use iced_native::text;

use crate::{hit_byte_index, offset_x_of_index, rope_ext::RopeExt, Rope};

/// The cursor of a text input.
#[derive(Debug, Copy, Clone)]
pub struct Cursor {
	state: State,
	offset_x_hint: Option<f32>,
}

/// The state of a [`Cursor`].
#[derive(Debug, Copy, Clone)]
pub enum State {
	/// Cursor without a selection
	Index(usize),

	/// Cursor selecting a range of text
	Selection {
		/// The start of the selection
		start: usize,
		/// The end of the selection
		end: usize,
	},
}

impl Default for Cursor {
	fn default() -> Self {
		Cursor {
			state: State::Index(0),
			offset_x_hint: None,
		}
	}
}

impl Cursor {
	/// Returns the [`State`] of the [`Cursor`].
	pub fn state(&self, value: &Rope) -> State {
		match self.state {
			State::Index(index) => State::Index(index.min(value.len_bytes())),
			State::Selection { start, end } => {
				let start = start.min(value.len_bytes());
				let end = end.min(value.len_bytes());

				if start == end {
					State::Index(start)
				} else {
					State::Selection { start, end }
				}
			}
		}
	}

	/// Returns the current selection of the [`Cursor`] for the given [`Value`].
	///
	/// `start` is guaranteed to be <= than `end`.
	pub fn selection(&self, value: &Rope) -> Option<(usize, usize)> {
		match self.state(value) {
			State::Selection { start, end } => {
				Some((start.min(end), start.max(end)))
			}
			_ => None,
		}
	}

	pub(crate) fn move_to_byte(&mut self, position: usize) {
		self.move_to_impl(position);
		self.offset_x_hint = None;
	}

	fn move_to_impl(&mut self, position: usize) {
		self.state = State::Index(position);
	}

	pub(crate) fn move_right(&mut self, value: &Rope) {
		let index = match self.state(value) {
			State::Selection { start, end } => {
				self.move_to_byte(end.max(start));
				return;
			}
			State::Index(index) => index,
		};

		self.move_to_byte(value.next_grapheme(index));
	}

	pub(crate) fn move_right_by_words(&mut self, value: &Rope) {
		self.move_to_impl(value.next_end_of_word(self.end(value)));
		self.offset_x_hint = None;
	}

	pub(crate) fn move_right_by_line(&mut self, value: &Rope) {
		self.move_to_impl(find_end_of_line(self.end(value), value));
		self.offset_x_hint = None;
	}

	pub(crate) fn move_right_by_bytes(&mut self, value: &Rope, amount: usize) {
		match self.state(value) {
			State::Index(index) => self.move_to_impl(
				index.saturating_add(amount).min(value.len_bytes()),
			),
			State::Selection { start, end } => {
				self.move_to_impl(end.max(start))
			}
		}
		self.offset_x_hint = None;
	}

	pub(crate) fn move_left(&mut self, value: &Rope) {
		let index = match self.state(value) {
			State::Selection { start, end } => {
				self.move_to_byte(start.min(end));
				return;
			}
			State::Index(index) if index > 0 => index,
			_ => {
				self.move_to_byte(0);
				return;
			}
		};

		self.move_to_byte(value.previous_grapheme(index));
	}

	pub(crate) fn move_left_by_words(&mut self, value: &Rope) {
		self.move_to_impl(value.previous_start_of_word(self.start(value)));
		self.offset_x_hint = None;
	}

	pub(crate) fn move_left_by_line(&mut self, value: &Rope) {
		self.move_to_byte(
			value.line_to_byte(value.byte_to_line(self.start(value))),
		);
	}

	pub(crate) fn move_up<Renderer>(
		&mut self,
		value: &Rope,
		renderer: &Renderer,
		font: Renderer::Font,
		tab_width: u8,
	) where
		Renderer: text::Renderer,
	{
		match self.state(value) {
			State::Index(index) if index > 0 => {
				let (new_index, offset_x) = find_index_above(
					index,
					self.offset_x_hint,
					value,
					renderer,
					font,
					tab_width,
				);
				self.move_to_impl(new_index);
				self.offset_x_hint = Some(offset_x);
			}
			State::Selection { start, end } => {
				let (new_index, offset_x) = find_index_above(
					start.min(end),
					self.offset_x_hint,
					value,
					renderer,
					font,
					tab_width,
				);
				self.move_to_impl(new_index);
				self.offset_x_hint = Some(offset_x);
			}
			_ => self.move_to_impl(0),
		}
	}

	pub(crate) fn move_down<Renderer>(
		&mut self,
		value: &Rope,
		renderer: &Renderer,
		font: Renderer::Font,
		tab_width: u8,
	) where
		Renderer: text::Renderer,
	{
		match self.state(value) {
			State::Index(index) => {
				let (new_index, offset_x) = find_index_below(
					index,
					self.offset_x_hint,
					value,
					renderer,
					font,
					tab_width,
				);
				self.move_to_impl(new_index);
				self.offset_x_hint = Some(offset_x);
			}
			State::Selection { start, end } => {
				let (new_index, offset_x) = find_index_below(
					end.max(start),
					self.offset_x_hint,
					value,
					renderer,
					font,
					tab_width,
				);
				self.move_to_impl(new_index);
				self.offset_x_hint = Some(offset_x);
			}
		}
	}

	fn select_range_impl(&mut self, start: usize, end: usize) {
		if start == end {
			self.state = State::Index(start);
		} else {
			self.state = State::Selection { start, end };
		}
	}

	pub(crate) fn select_range(&mut self, start: usize, end: usize) {
		self.select_range_impl(start, end);
		self.offset_x_hint = None;
	}

	pub(crate) fn select_left(&mut self, value: &Rope) {
		match self.state(value) {
			State::Index(index) if index > 0 => {
				self.select_range_impl(index, value.previous_grapheme(index));
			}
			State::Selection { start, end } if end > 0 => {
				self.select_range_impl(start, value.previous_grapheme(end));
			}
			_ => {}
		}
		self.offset_x_hint = None;
	}

	pub(crate) fn select_right(&mut self, value: &Rope) {
		match self.state(value) {
			State::Index(index) if index < value.len_bytes() => {
				self.select_range_impl(index, value.next_grapheme(index));
			}
			State::Selection { start, end } if end < value.len_bytes() => {
				self.select_range_impl(start, value.next_grapheme(end));
			}
			_ => {}
		}
		self.offset_x_hint = None;
	}

	pub(crate) fn select_left_by_words(&mut self, value: &Rope) {
		match self.state(value) {
			State::Index(index) => self
				.select_range_impl(index, value.previous_start_of_word(index)),
			State::Selection { start, end } => {
				self.select_range_impl(start, value.previous_start_of_word(end))
			}
		}
		self.offset_x_hint = None;
	}

	pub(crate) fn select_right_by_words(&mut self, value: &Rope) {
		match self.state(value) {
			State::Index(index) => {
				self.select_range_impl(index, value.next_end_of_word(index));
			}
			State::Selection { start, end } => {
				self.select_range_impl(start, value.next_end_of_word(end));
			}
		}
		self.offset_x_hint = None;
	}

	pub(crate) fn select_left_by_line(&mut self, value: &Rope) {
		match self.state(value) {
			State::Index(index) => {
				let line_index = value.byte_to_line(index);
				self.select_range_impl(index, value.line_to_byte(line_index));
			}
			State::Selection { start, end } => {
				let line_index = value.byte_to_line(end);
				self.select_range_impl(start, value.line_to_byte(line_index));
			}
		}
		self.offset_x_hint = None;
	}

	pub(crate) fn select_right_by_line(&mut self, value: &Rope) {
		match self.state(value) {
			State::Index(index) => {
				self.select_range_impl(index, find_end_of_line(index, value));
			}
			State::Selection { start, end } => {
				self.select_range_impl(start, find_end_of_line(end, value));
			}
		}
		self.offset_x_hint = None;
	}

	pub(crate) fn select_up<Renderer>(
		&mut self,
		value: &Rope,
		renderer: &Renderer,
		font: Renderer::Font,
		tab_width: u8,
	) where
		Renderer: text::Renderer,
	{
		match self.state(value) {
			State::Index(index) if index > 0 => {
				let (above, offset_x) = find_index_above(
					index,
					self.offset_x_hint,
					value,
					renderer,
					font,
					tab_width,
				);
				self.select_range_impl(index, above);
				self.offset_x_hint = Some(offset_x);
			}
			State::Selection { start, end } if end > 0 => {
				let (above, offset_x) = find_index_above(
					end,
					self.offset_x_hint,
					value,
					renderer,
					font,
					tab_width,
				);
				self.select_range_impl(start, above);
				self.offset_x_hint = Some(offset_x);
			}
			_ => {}
		}
	}

	pub(crate) fn select_down<Renderer>(
		&mut self,
		value: &Rope,
		renderer: &Renderer,
		font: Renderer::Font,
		tab_width: u8,
	) where
		Renderer: text::Renderer,
	{
		match self.state(value) {
			State::Index(index) if index < value.len_bytes() => {
				let (below, offset_x) = find_index_below(
					index,
					self.offset_x_hint,
					value,
					renderer,
					font,
					tab_width,
				);
				self.select_range_impl(index, below);
				self.offset_x_hint = Some(offset_x);
			}
			State::Selection { start, end } if end < value.len_bytes() => {
				let (below, offset_x) = find_index_below(
					end,
					self.offset_x_hint,
					value,
					renderer,
					font,
					tab_width,
				);
				self.select_range_impl(start, below);
				self.offset_x_hint = Some(offset_x);
			}
			_ => {}
		}
	}

	pub(crate) fn select_all(&mut self, value: &Rope) {
		self.select_range_impl(0, value.len_bytes());
		self.offset_x_hint = None;
	}

	pub(crate) fn start(&self, value: &Rope) -> usize {
		let start = match self.state {
			State::Index(index) => index,
			State::Selection { start, .. } => start,
		};

		start.min(value.len_bytes())
	}

	pub(crate) fn end(&self, value: &Rope) -> usize {
		let end = match self.state {
			State::Index(index) => index,
			State::Selection { end, .. } => end,
		};

		end.min(value.len_bytes())
	}
}

fn find_end_of_line(index: usize, value: &Rope) -> usize {
	let next_line_start = value.line_to_byte(value.byte_to_line(index) + 1);

	let last_byte = next_line_start.checked_sub(1).map(|i| value.byte(i));
	let second_last_byte =
		next_line_start.checked_sub(2).map(|i| value.byte(i));

	if last_byte == Some(b'\n') && second_last_byte == Some(b'\r') {
		next_line_start - 2
	} else {
		next_line_start - 1
	}
}

fn find_index_above<Renderer>(
	index: usize,
	offset_x_hint: Option<f32>,
	value: &Rope,
	renderer: &Renderer,
	font: Renderer::Font,
	tab_width: u8,
) -> (usize, f32)
where
	Renderer: text::Renderer,
{
	let line_index = value.byte_to_line(index);
	if line_index == 0 {
		return (0, 0.0);
	}

	let offset_x = match offset_x_hint {
		Some(x) => x,
		None => offset_x_of_index(
			index,
			value,
			renderer,
			font.clone(),
			None,
			tab_width,
		),
	};

	let previous_line_start = value.line_to_byte(line_index - 1);
	let previous_line = value.line(line_index - 1);

	{
		let mut bytes = previous_line.bytes();
		match (bytes.next(), bytes.next()) {
			(None, _)
			| (Some(b'\n'), _)
			| (Some(b'\r'), None | Some(b'\n')) => {
				return (previous_line_start, offset_x)
			}
			_ => (),
		}
	}

	let size = renderer.default_size();

	let index_above = hit_byte_index(
		renderer,
		previous_line,
		None,
		size,
		font,
		tab_width,
		Point::new(offset_x, f32::from(size) / 2.0),
	)
	.map_or_else(
		|| {
			panic!(
				"Failed to hit test for point ({offset_x}, {})",
				f32::from(size) / 2.0
			)
		},
		|offset| previous_line_start + offset,
	);

	(index_above, offset_x)
}

fn find_index_below<Renderer>(
	index: usize,
	offset_x_hint: Option<f32>,
	value: &Rope,
	renderer: &Renderer,
	font: Renderer::Font,
	tab_width: u8,
) -> (usize, f32)
where
	Renderer: text::Renderer,
{
	let line_index = value.byte_to_line(index);
	if line_index + 1 == value.len_lines() {
		return (
			value.len_bytes(),
			offset_x_of_index(
				value.len_bytes(),
				value,
				renderer,
				font,
				None,
				tab_width,
			),
		);
	}

	let offset_x = match offset_x_hint {
		Some(x) => x,
		None => offset_x_of_index(
			index,
			value,
			renderer,
			font.clone(),
			None,
			tab_width,
		),
	};

	let next_line_start = value.line_to_byte(line_index + 1);
	let next_line = value.line(line_index + 1);

	{
		let mut bytes = next_line.bytes();
		match (bytes.next(), bytes.next()) {
			(None, _)
			| (Some(b'\n'), _)
			| (Some(b'\r'), None | Some(b'\n')) => return (next_line_start, offset_x),
			_ => (),
		}
	}

	let size = renderer.default_size();

	let index_below = hit_byte_index(
		renderer,
		next_line,
		None,
		size,
		font,
		tab_width,
		Point::new(offset_x, f32::from(size) / 2.0),
	)
	.map_or_else(
		|| {
			panic!(
				"Failed to hit test for point ({offset_x}, {})",
				f32::from(size) / 2.0
			)
		},
		|offset| next_line_start + offset,
	);

	(index_below, offset_x)
}
