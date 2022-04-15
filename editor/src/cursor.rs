//! Track the cursor of a text input.
use iced_graphics::{Point, Size};
use iced_native::text;

use crate::Value;

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
	pub fn state(&self, value: &Value) -> State {
		match self.state {
			State::Index(index) => State::Index(index.min(value.len())),
			State::Selection { start, end } => {
				let start = start.min(value.len());
				let end = end.min(value.len());

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
	pub fn selection(&self, value: &Value) -> Option<(usize, usize)> {
		match self.state(value) {
			State::Selection { start, end } => {
				Some((start.min(end), start.max(end)))
			}
			_ => None,
		}
	}

	pub(crate) fn move_to(&mut self, position: usize) {
		self.move_to_impl(position);
		self.offset_x_hint = None;
	}

	fn move_to_impl(&mut self, position: usize) {
		self.state = State::Index(position);
	}

	pub(crate) fn move_right(&mut self, value: &Value) {
		self.move_right_by_amount(value, 1);
	}

	pub(crate) fn move_right_by_words(&mut self, value: &Value) {
		self.move_to_impl(value.next_end_of_word(self.end(value)));
		self.offset_x_hint = None;
	}

	pub(crate) fn move_right_by_line(&mut self, value: &Value) {
		self.move_to_impl(value.next_end_of_line(self.end(value)));
		self.offset_x_hint = None;
	}

	pub(crate) fn move_right_by_amount(
		&mut self,
		value: &Value,
		amount: usize,
	) {
		match self.state(value) {
			State::Index(index) => {
				self.move_to_impl(index.saturating_add(amount).min(value.len()))
			}
			State::Selection { start, end } => {
				self.move_to_impl(end.max(start))
			}
		}
		self.offset_x_hint = None;
	}

	pub(crate) fn move_left(&mut self, value: &Value) {
		match self.state(value) {
			State::Index(index) if index > 0 => self.move_to_impl(index - 1),
			State::Selection { start, end } => {
				self.move_to_impl(start.min(end))
			}
			_ => self.move_to_impl(0),
		}
		self.offset_x_hint = None;
	}

	pub(crate) fn move_left_by_words(&mut self, value: &Value) {
		self.move_to_impl(value.previous_start_of_word(self.start(value)));
		self.offset_x_hint = None;
	}

	pub(crate) fn move_left_by_line(&mut self, value: &Value) {
		self.move_to_impl(value.previous_start_of_line(self.start(value)));
		self.offset_x_hint = None;
	}

	pub(crate) fn move_up<Renderer>(
		&mut self,
		value: &Value,
		renderer: &Renderer,
		font: Renderer::Font,
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
				);
				self.move_to_impl(new_index);
				self.offset_x_hint = Some(offset_x);
			}
			_ => self.move_to_impl(0),
		}
	}

	pub(crate) fn move_down<Renderer>(
		&mut self,
		value: &Value,
		renderer: &Renderer,
		font: Renderer::Font,
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

	pub(crate) fn select_left(&mut self, value: &Value) {
		match self.state(value) {
			State::Index(index) if index > 0 => {
				self.select_range_impl(index, index - 1)
			}
			State::Selection { start, end } if end > 0 => {
				self.select_range_impl(start, end - 1)
			}
			_ => {}
		}
		self.offset_x_hint = None;
	}

	pub(crate) fn select_right(&mut self, value: &Value) {
		match self.state(value) {
			State::Index(index) if index < value.len() => {
				self.select_range_impl(index, index + 1)
			}
			State::Selection { start, end } if end < value.len() => {
				self.select_range_impl(start, end + 1)
			}
			_ => {}
		}
		self.offset_x_hint = None;
	}

	pub(crate) fn select_left_by_words(&mut self, value: &Value) {
		match self.state(value) {
			State::Index(index) => self
				.select_range_impl(index, value.previous_start_of_word(index)),
			State::Selection { start, end } => {
				self.select_range_impl(start, value.previous_start_of_word(end))
			}
		}
		self.offset_x_hint = None;
	}

	pub(crate) fn select_right_by_words(&mut self, value: &Value) {
		match self.state(value) {
			State::Index(index) => {
				self.select_range_impl(index, value.next_end_of_word(index))
			}
			State::Selection { start, end } => {
				self.select_range_impl(start, value.next_end_of_word(end))
			}
		}
		self.offset_x_hint = None;
	}

	pub(crate) fn select_left_by_line(&mut self, value: &Value) {
		match self.state(value) {
			State::Index(index) => self
				.select_range_impl(index, value.previous_start_of_line(index)),
			State::Selection { start, end } => {
				self.select_range_impl(start, value.previous_start_of_line(end))
			}
		}
		self.offset_x_hint = None;
	}

	pub(crate) fn select_right_by_line(&mut self, value: &Value) {
		match self.state(value) {
			State::Index(index) => {
				self.select_range_impl(index, value.next_end_of_line(index))
			}
			State::Selection { start, end } => {
				self.select_range_impl(start, value.next_end_of_line(end))
			}
		}
		self.offset_x_hint = None;
	}

	pub(crate) fn select_up<Renderer>(
		&mut self,
		value: &Value,
		renderer: &Renderer,
		font: Renderer::Font,
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
				);
				self.select_range_impl(start, above);
				self.offset_x_hint = Some(offset_x);
			}
			_ => {}
		}
	}

	pub(crate) fn select_down<Renderer>(
		&mut self,
		value: &Value,
		renderer: &Renderer,
		font: Renderer::Font,
	) where
		Renderer: text::Renderer,
	{
		match self.state(value) {
			State::Index(index) if index < value.len() => {
				let (below, offset_x) = find_index_below(
					index,
					self.offset_x_hint,
					value,
					renderer,
					font,
				);
				self.select_range_impl(index, below);
				self.offset_x_hint = Some(offset_x);
			}
			State::Selection { start, end } if end < value.len() => {
				let (below, offset_x) = find_index_below(
					end,
					self.offset_x_hint,
					value,
					renderer,
					font,
				);
				self.select_range_impl(start, below);
				self.offset_x_hint = Some(offset_x);
			}
			_ => {}
		}
	}

	pub(crate) fn select_all(&mut self, value: &Value) {
		self.select_range_impl(0, value.len());
		self.offset_x_hint = None;
	}

	pub(crate) fn start(&self, value: &Value) -> usize {
		let start = match self.state {
			State::Index(index) => index,
			State::Selection { start, .. } => start,
		};

		start.min(value.len())
	}

	pub(crate) fn end(&self, value: &Value) -> usize {
		let end = match self.state {
			State::Index(index) => index,
			State::Selection { end, .. } => end,
		};

		end.min(value.len())
	}
}

fn offset_x_of_index<Renderer>(
	index: usize,
	value: &Value,
	renderer: &Renderer,
	font: Renderer::Font,
) -> f32
where
	Renderer: text::Renderer,
{
	let line_start = value.previous_start_of_line(index);
	width_of_range(line_start, index, value, renderer, font)
}

fn width_of_range<Renderer>(
	start: usize,
	end: usize,
	value: &Value,
	renderer: &Renderer,
	font: Renderer::Font,
) -> f32
where
	Renderer: text::Renderer,
{
	let range = value.select(start, end);
	renderer.measure_width(&range.to_string(), renderer.default_size(), font)
}

fn find_index_above<Renderer>(
	index: usize,
	offset_x_hint: Option<f32>,
	value: &Value,
	renderer: &Renderer,
	font: Renderer::Font,
) -> (usize, f32)
where
	Renderer: text::Renderer,
{
	let line_start = value.previous_start_of_line(index);

	if line_start == 0 {
		return (0, 0.0);
	}

	let previous_line_end = line_start - 1;

	let previous_line_start = value.previous_start_of_line(previous_line_end);

	let offset_x = match offset_x_hint {
		Some(x) => x,
		None => {
			width_of_range(line_start, index, value, renderer, font.clone())
		}
	};

	if previous_line_start == previous_line_end {
		return (previous_line_start, offset_x);
	}

	let previous_line = value.select(previous_line_start, previous_line_end);

	let size = renderer.default_size().into();

	let index_above = renderer
		.hit_test(
			&previous_line.to_string(),
			size,
			font,
			Size::INFINITY,
			Point::new(offset_x, size / 2.0),
			true,
		)
		.map(text::Hit::cursor)
		.map(|offset| previous_line_start + offset)
		.unwrap_or_else(|| {
			panic!("Failed to hit test for point ({offset_x}, {})", size / 2.0)
		});

	(index_above, offset_x)
}

fn find_index_below<Renderer>(
	index: usize,
	offset_x_hint: Option<f32>,
	value: &Value,
	renderer: &Renderer,
	font: Renderer::Font,
) -> (usize, f32)
where
	Renderer: text::Renderer,
{
	let next_line_start = value.next_end_of_line(index) + 1;

	if next_line_start >= value.len() {
		return (
			value.len(),
			offset_x_of_index(value.len(), value, renderer, font),
		);
	}

	let next_line_end = value.next_end_of_line(next_line_start);

	let offset_x = match offset_x_hint {
		Some(x) => x,
		None => offset_x_of_index(index, value, renderer, font.clone()),
	};

	if next_line_start == next_line_end {
		return (next_line_start, offset_x);
	}

	let next_line = value.select(next_line_start, next_line_end);

	let size = renderer.default_size().into();

	let index_below = renderer
		.hit_test(
			&next_line.to_string(),
			size,
			font,
			Size::INFINITY,
			Point::new(offset_x, size / 2.0),
			true,
		)
		.map(text::Hit::cursor)
		.map(|offset| next_line_start + offset)
		.unwrap_or_else(|| {
			panic!("Failed to hit test for point ({offset_x}, {})", size / 2.0)
		});

	(index_below, offset_x)
}
