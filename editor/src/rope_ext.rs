use std::borrow::Cow;

use ropey::{iter::Lines, Rope, RopeSlice};
use unicode_segmentation::{
	GraphemeCursor, GraphemeIncomplete, UnicodeSegmentation,
};

pub(crate) trait RopeExt {
	fn display(&self, tab_width: u8) -> RopeDisplay<'_>;

	fn next_end_of_word(&self, byte_index: usize) -> usize;

	fn previous_start_of_word(&self, byte_index: usize) -> usize;

	fn next_grapheme(&self, byte_index: usize) -> usize;

	fn previous_grapheme(&self, byte_index: usize) -> usize;
}

impl RopeExt for RopeSlice<'_> {
	fn display(&self, tab_width: u8) -> RopeDisplay<'_> {
		RopeDisplay {
			lines: self.lines(),
			tab_width,
		}
	}

	fn next_end_of_word(&self, byte_index: usize) -> usize {
		let line_index = self.byte_to_line(byte_index);
		let next_line_start = self.line_to_byte(line_index + 1);
		let line_end = if next_line_start == self.len_bytes() {
			self.len_bytes()
		} else {
			next_line_start - 1
		};

		if byte_index == line_end {
			return next_line_start;
		}

		let line = self.byte_slice(byte_index..line_end).to_string();

		UnicodeSegmentation::split_word_bound_indices(line.as_str())
			.find(|(_, word)| !word.trim_start().is_empty())
			.map(|(i, next_word)| byte_index + i + next_word.len())
			.unwrap_or_else(|| self.len_bytes())
	}

	fn previous_start_of_word(&self, byte_index: usize) -> usize {
		let line_index = self.byte_to_line(byte_index);
		let line_start = self.line_to_byte(line_index);

		if byte_index == line_start {
			return byte_index.saturating_sub(1);
		}

		let line = self.byte_slice(line_start..byte_index).to_string();

		UnicodeSegmentation::split_word_bound_indices(line.as_str())
			.filter(|(_, word)| !word.trim_start().is_empty())
			.next_back()
			.map(|(i, _)| line_start + i)
			.unwrap_or(0)
	}

	fn next_grapheme(&self, byte_index: usize) -> usize {
		let mut cursor =
			GraphemeCursor::new(byte_index, self.len_bytes(), true);

		let (mut chunks, mut chunk_start, ..) = self.chunks_at_byte(byte_index);

		let mut chunk = match chunks.next() {
			None => return self.len_bytes(),
			Some(c) => c,
		};

		loop {
			match cursor.next_boundary(chunk, chunk_start) {
				Ok(Some(index)) => break index,
				Ok(None) => break self.len_bytes(),
				Err(GraphemeIncomplete::PreContext(needed)) => {
					let (chunk, chunk_start, ..) = self.chunk_at_byte(needed);
					cursor.provide_context(chunk, chunk_start);
				}
				Err(GraphemeIncomplete::NextChunk) => {
					chunk = match chunks.next() {
						None => break self.len_bytes(),
						Some(c) => {
							chunk_start += chunk.len();
							c
						}
					};
				}
				Err(_) => {
					panic!(
						"Failed to get next grapheme boundary at {byte_index}"
					)
				}
			}
		}
	}

	fn previous_grapheme(&self, byte_index: usize) -> usize {
		if byte_index == self.len_bytes() && byte_index == 0 {
			return 0;
		}

		let mut cursor =
			GraphemeCursor::new(byte_index, self.len_bytes(), true);

		let (mut chunks, mut chunk_start, ..) = self.chunks_at_byte(byte_index);

		let next = if byte_index == self.len_bytes() {
			let next = chunks.prev();
			if let Some(c) = next {
				chunk_start -= c.len();
			}
			next
		} else {
			let next = chunks.next();
			chunks.prev();
			next
		};

		let mut chunk = match next {
			None => return 0,
			Some(c) => c,
		};

		loop {
			match cursor.prev_boundary(chunk, chunk_start) {
				Ok(Some(index)) => break index,
				Ok(None) => break 0,
				Err(GraphemeIncomplete::PreContext(needed)) => {
					let (chunk, chunk_start, ..) = self.chunk_at_byte(needed);
					cursor.provide_context(chunk, chunk_start);
				}
				Err(GraphemeIncomplete::PrevChunk) => {
					chunk = match chunks.prev() {
						None => break 0,
						Some(c) => {
							chunk_start -= c.len();
							c
						}
					};
				}
				Err(_) => {
					panic!(
						"Failed to get previous \
						grapheme boundary at {byte_index}"
					)
				}
			}
		}
	}
}

impl RopeExt for Rope {
	fn display(&self, tab_width: u8) -> RopeDisplay<'_> {
		RopeDisplay {
			lines: self.lines(),
			tab_width,
		}
	}

	fn next_end_of_word(&self, byte_index: usize) -> usize {
		self.byte_slice(..).next_end_of_word(byte_index)
	}

	fn previous_start_of_word(&self, byte_index: usize) -> usize {
		self.byte_slice(..).previous_start_of_word(byte_index)
	}

	fn next_grapheme(&self, byte_index: usize) -> usize {
		self.byte_slice(..).next_grapheme(byte_index)
	}

	fn previous_grapheme(&self, byte_index: usize) -> usize {
		self.byte_slice(..).previous_grapheme(byte_index)
	}
}

pub(crate) struct RopeDisplay<'r> {
	lines: Lines<'r>,
	tab_width: u8,
}

impl<'r> Iterator for RopeDisplay<'r> {
	type Item = Cow<'r, str>;

	fn next(&mut self) -> Option<Self::Item> {
		let line = self.lines.next()?;

		let mut chunks = line.chunks();

		let chunk = replace_tab(chunks.next()?, self.tab_width);

		match chunks.next() {
			None => Some(chunk),
			Some(next) => {
				let mut chunk = chunk.into_owned();
				chunk += &replace_tab(next, self.tab_width);
				chunk.extend(chunks.map(|c| replace_tab(c, self.tab_width)));
				Some(Cow::Owned(chunk))
			}
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.lines.size_hint()
	}
}

fn replace_tab(chunk: &str, tab_width: u8) -> Cow<'_, str> {
	// 255 spaces
	const SPACES: &str = "                                                                                                                                                                                                                                                               ";

	if chunk.as_bytes().contains(&b'\t') {
		chunk.replace('\t', &SPACES[..tab_width as usize]).into()
	} else {
		chunk.into()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn single_line_display() {
		let rope = Rope::from_str("hello world");
		assert_eq!(&["hello world"][..], rope.display(1).collect::<Vec<_>>());
	}

	#[test]
	fn multi_line_display() {
		let rope = Rope::from_str("hello\nworld");
		assert_eq!(
			&["hello\n", "world"][..],
			rope.display(1).collect::<Vec<_>>()
		);
	}

	#[test]
	fn display_empty_line() {
		let rope = Rope::from_str("hello\n\nworld");
		assert_eq!(
			&["hello\n", "\n", "world"][..],
			rope.display(1).collect::<Vec<_>>()
		);
	}

	#[test]
	fn display_with_tabs() {
		let rope = Rope::from_str("\t\thello\n\tworld");
		assert_eq!(
			&["    hello\n", "  world"][..],
			rope.display(2).collect::<Vec<_>>()
		);
	}

	#[test]
	fn next_grapheme() {
		let rope = Rope::from_str("bye 💔 :(");
		assert_eq!(rope.next_grapheme(0), 1);
		assert_eq!(rope.next_grapheme(1), 2);
		assert_eq!(rope.next_grapheme(4), 8);
		assert_eq!(rope.next_grapheme(10), 11);
		assert_eq!(rope.next_grapheme(11), 11);
	}

	#[test]
	fn previous_grapheme() {
		let rope = Rope::from_str("bye 💔 :(");
		assert_eq!(rope.previous_grapheme(11), 10);
		assert_eq!(rope.previous_grapheme(9), 8);
		assert_eq!(rope.previous_grapheme(8), 4);
		assert_eq!(rope.previous_grapheme(4), 3);
		assert_eq!(rope.previous_grapheme(0), 0);
	}
}
