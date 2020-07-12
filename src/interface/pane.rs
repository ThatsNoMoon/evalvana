use super::UpdatingContext;

use crate::{
	events::{actions::Action, Event},
	geometry::ScreenPixelRect,
	rendering::drawing::DrawingId,
	repl::evaluation::{EditedExpression, Evaluation},
};

use winit::event::{Event as WinitEvent, WindowEvent};

#[derive(Debug, PartialEq, Eq)]
pub struct Pane {
	pub name: String,
	pub evaluations: Vec<Evaluation>,
	pub current_input: EditedExpression,
	pub drawing_id: DrawingId,
	pub drawn_bounds: Option<ScreenPixelRect>,
}

impl Pane {
	pub fn new(ctx: &mut UpdatingContext<'_>, name: String) -> Pane {
		Pane {
			name,
			evaluations: vec![],
			current_input: EditedExpression::default(),
			drawing_id: ctx.drawing_manager.next_drawing_id(),
			drawn_bounds: None,
		}
	}

	pub fn update(&mut self, ctx: &mut UpdatingContext<'_>) -> Action {
		match &ctx.event {
			Event::WinitEvent(WinitEvent::WindowEvent {
				event: WindowEvent::Resized(_),
				..
			}) => {
				self.drawn_bounds = None;
				Action::none()
			}

			_ => Action::none(),
		}
	}
}
