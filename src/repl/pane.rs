use super::evaluation::{
	Evaluation,
	EditedExpression,
};

use crate::input::commands::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pane {
	name: String,
	evaluations: Vec<Evaluation>,
	current_input: EditedExpression,
}

impl Pane {
	pub fn handle_input(&mut self, input: Command) {
		self.current_input.handle_input(input);
	}
}