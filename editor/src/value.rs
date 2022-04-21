use std::cmp::Ordering;

use unicode_segmentation::UnicodeSegmentation;

/// The value of a [`TextInput`].
///
/// [`TextInput`]: crate::widget::TextInput
// TODO: Reduce allocations, cache results (?)
#[derive(Debug, Clone)]
pub struct Value {
	graphemes: Vec<String>,
}

impl Value {
	/// Creates a new [`Value`] from a string slice.
	pub fn new(string: &str) -> Self {
		let graphemes = UnicodeSegmentation::graphemes(string, true)
			.map(String::from)
			.collect();

		Self { graphemes }
	}

	/// Returns whether the [`Value`] is empty or not.
	///
	/// A [`Value`] is empty when it contains no graphemes.
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Returns the total amount of graphemes in the [`Value`].
	pub fn len(&self) -> usize {
		self.graphemes.len()
	}

	pub fn lines(
		&self,
		tab_width: impl Into<Option<u8>>,
	) -> impl Iterator<Item = String> + '_ {
		let mut start = 0;
		let tab_width = tab_width.into();

		std::iter::from_fn(move || -> Option<String> {
			match start.cmp(&self.graphemes.len()) {
				Ordering::Greater => None,
				Ordering::Equal => {
					start += 1;
					Some(String::new())
				}
				Ordering::Less => {
					let end = self.next_end_of_line(start);
					let slice =
						&self.graphemes[start..=end.min(self.len() - 1)];
					start = end + 1;
					match tab_width {
						Some(n) => Some(
							slice.iter().map(|g| replace_tab(g, n)).collect(),
						),
						None => Some(slice.concat()),
					}
				}
			}
		})
	}

	/// Returns the number of line endings in the [`Value`].
	pub fn count_lines(&self) -> usize {
		self.graphemes
			.iter()
			.filter(|&g| g == "\n" || g == "\r\n")
			.count()
	}

	/// Returns the number of line endings before the given index in the [`Value`].
	pub fn count_lines_before(&self, index: usize) -> usize {
		self.graphemes[..index.min(self.len())]
			.iter()
			.filter(|&g| g == "\n" || g == "\r\n")
			.count()
	}

	/// Returns the number of line endings between the two indices in the [`Value`].
	pub fn count_lines_between(&self, start: usize, end: usize) -> usize {
		self.graphemes[start..end.min(self.len())]
			.iter()
			.filter(|&g| g == "\n" || g == "\r\n")
			.count()
	}

	/// Returns the `n`th line in the [`Value`].
	pub fn nth_line_start(&self, n: usize) -> Option<usize> {
		if n == 0 {
			Some(0)
		} else {
			self.graphemes
				.iter()
				.enumerate()
				.filter(|&(_, g)| g == "\n" || g == "\r\n")
				.nth(n - 1)
				.map(|(i, _)| i + 1)
				.filter(|&i| i < self.graphemes.len())
		}
	}

	/// Returns the number of tabs between the two indices in the [`Value`].
	pub fn count_tabs_between(&self, start: usize, end: usize) -> usize {
		self.graphemes[start..end.min(self.len())]
			.iter()
			.filter(|&g| g == "\t")
			.count()
	}

	/// Returns the position of the previous start of a word from the given
	/// grapheme `index`.
	pub fn previous_start_of_word(&self, index: usize) -> usize {
		let previous_string =
			&self.graphemes[..index.min(self.graphemes.len())].concat();

		UnicodeSegmentation::split_word_bound_indices(previous_string as &str)
			.filter(|(_, word)| !word.trim_start().is_empty())
			.next_back()
			.map(|(i, previous_word)| {
				index
					- UnicodeSegmentation::graphemes(previous_word, true)
						.count() - UnicodeSegmentation::graphemes(
					&previous_string[i + previous_word.len()..] as &str,
					true,
				)
				.count()
			})
			.unwrap_or(0)
	}

	/// Returns the position of the next end of a word from the given grapheme
	/// `index`.
	pub fn next_end_of_word(&self, index: usize) -> usize {
		let next_string = &self.graphemes[index..].concat();

		UnicodeSegmentation::split_word_bound_indices(next_string as &str)
			.find(|(_, word)| !word.trim_start().is_empty())
			.map(|(i, next_word)| {
				index
					+ UnicodeSegmentation::graphemes(next_word, true).count()
					+ UnicodeSegmentation::graphemes(
						&next_string[..i] as &str,
						true,
					)
					.count()
			})
			.unwrap_or(self.len())
	}

	/// Returns the position of the previous start of a line from the given
	/// grapheme `index`.
	pub fn previous_start_of_line(&self, index: usize) -> usize {
		self.graphemes[..index.min(self.len())]
			.iter()
			.rposition(|g| g == "\n" || g == "\r\n")
			.map_or(0, |x| (x + 1).min(self.len()))
	}

	/// Returns the position of the next end of a line from the given grapheme
	/// `index`.
	pub fn next_end_of_line(&self, index: usize) -> usize {
		if index == self.len() {
			return self.len();
		}

		self.graphemes[index..]
			.iter()
			.position(|g| g == "\n" || g == "\r\n")
			.map_or(self.len(), |x| x + index)
	}

	/// Returns a new [`Value`] containing the graphemes from `start` until the
	/// given `end`.
	pub fn select(&self, start: usize, end: usize) -> Self {
		let graphemes =
			self.graphemes[start.min(self.len())..end.min(self.len())].to_vec();

		Self { graphemes }
	}

	/// Returns a new [`Value`] containing the graphemes until the given
	/// `index`.
	pub fn until(&self, index: usize) -> Self {
		let graphemes = self.graphemes[..index.min(self.len())].to_vec();

		Self { graphemes }
	}

	/// Returns a new [`Value`] containing the graphemes after and including the
	/// given `index`.
	pub fn after(&self, index: usize) -> Self {
		let graphemes = self.graphemes[index.min(self.len())..].to_vec();

		Self { graphemes }
	}

	/// Returns a new [`Value`] containing the graphemes after the last
	/// occurence of the given grapheme.
	///
	/// Returns `None` if the given grapheme does not appear in this [`Value`].
	pub fn after_grapheme(&self, grapheme: &str) -> Option<Self> {
		let start = self.graphemes.iter().rposition(|g| g == grapheme)?;

		Some(self.after(start))
	}

	/// Inserts a new `char` at the given grapheme `index`.
	pub fn insert(&mut self, index: usize, c: char) {
		self.graphemes.insert(index, c.to_string());

		self.graphemes =
			UnicodeSegmentation::graphemes(self.to_string(None).as_str(), true)
				.map(String::from)
				.collect();
	}

	/// Inserts a bunch of graphemes at the given grapheme `index`.
	pub fn insert_many(&mut self, index: usize, mut value: Value) {
		let _ = self
			.graphemes
			.splice(index..index, value.graphemes.drain(..));
	}

	/// Removes the grapheme at the given `index`.
	pub fn remove(&mut self, index: usize) {
		let _ = self.graphemes.remove(index);
	}

	/// Removes the graphemes from `start` to `end`.
	pub fn remove_many(&mut self, start: usize, end: usize) {
		let _ = self.graphemes.splice(start..end, std::iter::empty());
	}

	/// Gets the content of this [`Value`] with the specified tab width.
	///
	/// Passing `None` will disable conversion of tabs into spaces. Otherwise,
	/// tabs will be converted into the given number of spaces.
	pub fn to_string(&self, tab_width: impl Into<Option<u8>>) -> String {
		match tab_width.into() {
			Some(n) => {
				self.graphemes.iter().map(|g| replace_tab(g, n)).collect()
			}
			None => self.graphemes.concat(),
		}
	}
}

fn replace_tab(grapheme: &str, tab_width: u8) -> &str {
	// 255 spaces
	const SPACES: &str = "                                                                                                                                                                                                                                                               ";

	if grapheme == "\t" {
		&SPACES[..tab_width as usize]
	} else {
		&*grapheme
	}
}
