use crate::input::commands::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditedExpression {
	input: String,
	cursor: usize,
}

impl EditedExpression {
	pub fn handle_input(&mut self, input: Command) {
		match input {
			Command::Insert(inserted) => {
				self.input.push(inserted);
				self.cursor += inserted.len_utf8();
			}
			Command::Backspace => {
				if let Some(removed) = self.input.pop() {
					self.cursor -= removed.len_utf8();
				}
			}
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expression {
	input: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Result {
	Empty,
	Success(PlainResult),
	Error(PlainResult),
	Compound(CompoundResult),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlainResult {
	text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundResult {
	success: Option<PlainResult>,
	warnings: Vec<PlainResult>,
	errors: Vec<PlainResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
	input: Expression,
	output: Result,
}