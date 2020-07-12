use crate::{
	events::{
		actions::{Action, ActionData},
		commands::Command,
		Event,
	},
	interface::UpdatingContext,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EditedExpression {
	pub input: String,
	pub cursor: usize,
}

impl EditedExpression {
	pub fn update(&mut self, ctx: &mut UpdatingContext) -> Action {
		match &ctx.event {
			Event::Command(Command::Insert(inserted)) => {
				self.input.push(*inserted);
				self.cursor += inserted.len_utf8();
				Action::Single(ActionData::RequestRedraw)
			}
			Event::Command(Command::Backspace) => match self.input.pop() {
				Some(removed) => {
					self.cursor -= removed.len_utf8();
					Action::Single(ActionData::RequestRedraw)
				}
				_ => Action::none(),
			},
			_ => Action::none(),
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
