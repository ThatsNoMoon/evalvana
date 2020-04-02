use crate::repl::evaluation::{
	Evaluation,
	EditedExpression,
};

use crate::input::commands::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pane {
	pub name: String,
	pub evaluations: Vec<Evaluation>,
	pub current_input: EditedExpression,
}

impl Pane {
	pub fn new(name: String) -> Pane {
		Pane {
			name,
			evaluations: vec![],
			current_input: EditedExpression::default(),
		}
	}
	
	pub fn handle_input(&mut self, input: Command) {
		self.current_input.handle_input(input);
	}
}