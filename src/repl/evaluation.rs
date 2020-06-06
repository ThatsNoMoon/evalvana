use crate::{
	events::{commands::Command, Event},
	interface::UpdatingContext,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EditedExpression {
	pub input: String,
	pub cursor: usize,
}

impl EditedExpression {
	pub fn update(&mut self, ctx: &mut UpdatingContext) {
		match &ctx.event {
			Event::Command(Command::Insert(inserted)) => {
				self.input.push(*inserted);
				self.cursor += inserted.len_utf8();
			}
			Event::Command(Command::Backspace) => {
				if let Some(removed) = self.input.pop() {
					self.cursor -= removed.len_utf8();
				}
			}
			_ => (),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expression {
	pub input: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Result {
	Empty,
	Success(PlainResult),
	Error(PlainResult),
	Warning(PlainResult),
	Compound(CompoundResult),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlainResult {
	pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundResult {
	pub success: Option<PlainResult>,
	pub warnings: Vec<PlainResult>,
	pub errors: Vec<PlainResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
	pub input: Expression,
	pub output: Result,
}
