use crate::{Cursor, Rope};

pub struct Editor<'a> {
	value: &'a mut Rope,
	cursor: &'a mut Cursor,
}

impl<'a> Editor<'a> {
	pub fn new(value: &'a mut Rope, cursor: &'a mut Cursor) -> Editor<'a> {
		Editor { value, cursor }
	}

	pub fn contents(&self) -> String {
		self.value.to_string()
	}

	pub fn insert(&mut self, character: char) {
		if let Some((left, right)) = self.cursor.selection(self.value) {
			self.cursor.move_left(self.value);
			let left = self.value.byte_to_char(left);
			let right = self.value.byte_to_char(right);
			self.value.remove(left..right);
		}

		self.value.insert_char(
			self.value.byte_to_char(self.cursor.end(self.value)),
			character,
		);
		self.cursor.move_right(self.value);
	}

	pub fn paste(&mut self, content: &str) {
		let length = content.len();

		if let Some((left, right)) = self.cursor.selection(self.value) {
			self.cursor.move_left(self.value);
			let left = self.value.byte_to_char(left);
			let right = self.value.byte_to_char(right);
			self.value.remove(left..right);
		}

		self.value.insert(
			self.value.byte_to_char(self.cursor.end(self.value)),
			content,
		);

		self.cursor.move_right_by_bytes(self.value, length);
	}

	pub fn backspace(&mut self) {
		match self.cursor.selection(self.value) {
			Some((start, end)) => {
				self.cursor.move_left(self.value);
				let start = self.value.byte_to_char(start);
				let end = self.value.byte_to_char(end);
				self.value.remove(start..end);
			}
			None => {
				let start = self.cursor.start(self.value);

				if start > 0 {
					self.cursor.move_left(self.value);
					self.value.remove(start - 1..start);
				}
			}
		}
	}

	pub fn delete(&mut self) {
		match self.cursor.selection(self.value) {
			Some(_) => {
				self.backspace();
			}
			None => {
				let end = self.cursor.end(self.value);

				if end < self.value.len_bytes() {
					self.value.remove(end..=end);
				}
			}
		}
	}
}
