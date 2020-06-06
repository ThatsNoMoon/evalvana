use super::{Pane, UpdatingContext};

use crate::{
	events::Event, geometry::ScreenPixelRect, rendering::drawing::DrawingId,
};

use winit::event::{Event as WinitEvent, WindowEvent};

#[derive(Debug, PartialEq, Eq)]
pub struct TreePane {
	pub pane_statuses: PaneStatuses,
	pub evaluators: Evaluators,
	pub drawing_id: DrawingId,
	pub drawn_bounds: Option<ScreenPixelRect>,
}

impl TreePane {
	pub fn new(ctx: &mut UpdatingContext<'_>) -> TreePane {
		TreePane {
			pane_statuses: PaneStatuses::default(),
			evaluators: Evaluators::default(),
			drawing_id: ctx.drawing_manager.next_drawing_id(),
			drawn_bounds: None,
		}
	}

	pub fn update(&mut self, ctx: &mut UpdatingContext<'_>) {
		match &ctx.event {
			Event::WinitEvent(WinitEvent::WindowEvent {
				event: WindowEvent::Resized(_),
				..
			}) => self.drawn_bounds = None,
			_ => (),
		}
	}
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PaneStatuses {
	pub pane_statuses: Vec<PaneStatus>,
	pub focused: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneStatus {
	pub name: String,
}

impl PaneStatus {
	pub fn of_pane(pane: &Pane) -> PaneStatus {
		PaneStatus {
			name: pane.name.clone(),
		}
	}
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Evaluators {
	pub evaluators: Vec<Evaluator>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluator {
	pub name: String,
}
