// Copyright 2020 Benjamin Scherer
// Licensed under the Open Software License version 3.0

use super::{Pane, UpdatingContext};

use crate::{
	events::{
		actions::{Action, ActionData},
		Event,
	},
	geometry::{ext::ScreenPixelPointExt, ScreenPixelPoint, ScreenPixelRect},
	rendering::drawing::DrawingId,
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

	pub fn update(&mut self, ctx: &mut UpdatingContext<'_>) -> Action {
		match &ctx.event {
			Event::WinitEvent(WinitEvent::WindowEvent {
				event: WindowEvent::Resized(_),
				..
			}) => self.drawn_bounds = None,
			_ => (),
		}
		let action =
			self.pane_statuses.update(ctx) + self.evaluators.update(ctx);
		if action.requests_redraw() {
			self.drawn_bounds = None;
		}
		action
	}
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PaneStatuses {
	pub pane_statuses: Vec<PaneStatus>,
	pub focused: u32,
	pub drawn_bounds: Option<ScreenPixelRect>,
}

impl PaneStatuses {
	fn update(&mut self, ctx: &mut UpdatingContext<'_>) -> Action {
		self.pane_statuses
			.iter_mut()
			.map(|status| status.update(ctx))
			.sum()
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneStatus {
	pub name: String,
	pub drawn_bounds: Option<ScreenPixelRect>,
	pub hovered: bool,
}

impl PaneStatus {
	pub fn of_pane(pane: &Pane) -> PaneStatus {
		PaneStatus {
			name: pane.name.clone(),
			drawn_bounds: None,
			hovered: false,
		}
	}

	fn update(&mut self, ctx: &mut UpdatingContext<'_>) -> Action {
		match &ctx.event {
			Event::WinitEvent(WinitEvent::WindowEvent {
				event: WindowEvent::CursorMoved { position, .. },
				..
			}) => match &self.drawn_bounds {
				Some(bounds) => {
					let position = ScreenPixelPoint::from_physical(
						*position,
						ctx.window.scale_factor(),
					);
					if !self.hovered && bounds.contains(position) {
						self.hovered = true;
						self.drawn_bounds = None;
						ActionData::RequestRedraw.into()
					} else if self.hovered && !bounds.contains(position) {
						self.hovered = false;
						self.drawn_bounds = None;
						ActionData::RequestRedraw.into()
					} else {
						Action::none()
					}
				}
				None => Action::none(),
			},
			_ => Action::none(),
		}
	}
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Evaluators {
	pub evaluators: Vec<Evaluator>,
	pub drawn_bounds: Option<ScreenPixelRect>,
}

impl Evaluators {
	fn update(&mut self, ctx: &mut UpdatingContext<'_>) -> Action {
		self.evaluators
			.iter_mut()
			.map(|evaluator| evaluator.update(ctx))
			.sum()
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluator {
	pub name: String,
	pub drawn_bounds: Option<ScreenPixelRect>,
	pub hovered: bool,
}

impl Evaluator {
	fn update(&mut self, ctx: &mut UpdatingContext<'_>) -> Action {
		match &ctx.event {
			Event::WinitEvent(WinitEvent::WindowEvent {
				event: WindowEvent::CursorMoved { position, .. },
				..
			}) => match &self.drawn_bounds {
				Some(bounds) => {
					let position = ScreenPixelPoint::from_physical(
						*position,
						ctx.window.scale_factor(),
					);
					if !self.hovered && bounds.contains(position) {
						self.hovered = true;
						self.drawn_bounds = None;
						Action::from(ActionData::RequestRedraw)
					} else if self.hovered && !bounds.contains(position) {
						self.hovered = false;
						self.drawn_bounds = None;
						Action::from(ActionData::RequestRedraw)
					} else {
						Action::none()
					}
				}
				None => Action::none(),
			},
			_ => Action::none(),
		}
	}
}
