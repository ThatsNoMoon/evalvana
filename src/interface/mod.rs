// Copyright 2020 Benjamin Scherer
// Licensed under the Open Software License version 3.0

mod pane;
pub use pane::Pane;
mod tree_pane;
pub use tree_pane::{
	Evaluator, Evaluators, PaneStatus, PaneStatuses, TreePane,
};

use crate::{
	events::{actions::Action, Event},
	geometry::{ScreenPixelRect, ScreenPixelSpace},
	rendering::drawing::{DrawingId, DrawingManager},
};

use euclid::Length;
use winit::{
	event::{Event as WinitEvent, WindowEvent},
	window::{Theme, Window},
};

#[derive(Debug)]
pub struct UpdatingContext<'a> {
	drawing_manager: &'a mut DrawingManager,
	pub window: &'a Window,
	pub event: Event<'a>,
}

impl<'a> UpdatingContext<'a> {
	pub fn new(
		drawing_manager: &'a mut DrawingManager,
		window: &'a Window,
		event: Event<'a>,
	) -> UpdatingContext<'a> {
		UpdatingContext {
			drawing_manager,
			window,
			event,
		}
	}
}

#[derive(Debug, PartialEq)]
pub enum InterfaceFocus {
	TreePane,
	Panes,
}

#[derive(Debug, PartialEq)]
pub struct Interface {
	pub panes: Panes,
	pub tree_pane: TreePane,
	pub tree_pane_width: Length<u32, ScreenPixelSpace>,
	pub theme: Theme,
	pub drawing_id: DrawingId,
	pub drawn_bounds: Option<ScreenPixelRect>,
	focus: InterfaceFocus,
}

impl Interface {
	pub fn new(ctx: &mut UpdatingContext<'_>) -> Interface {
		Interface {
			panes: Panes::default(),
			tree_pane: TreePane::new(ctx),
			tree_pane_width: Length::new(225),
			theme: Theme::Dark,
			drawing_id: ctx.drawing_manager.next_drawing_id(),
			drawn_bounds: None,
			focus: InterfaceFocus::Panes,
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

		let action = match &ctx.event {
			Event::Command(_) => match self.focus {
				InterfaceFocus::TreePane => self.tree_pane.update(ctx),
				InterfaceFocus::Panes => self.panes.update(ctx),
			},
			_ => self.tree_pane.update(ctx) + self.panes.update(ctx),
		};
		if action.requests_redraw() {
			self.drawn_bounds = None;
		}
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum Panes {
	VerticalSplit(PaneList),
	HorizontalSplit(PaneList),
	Tabbed(PaneList),
	Single(Pane),
}

impl Panes {
	pub fn title(&self) -> &str {
		match self {
			Panes::VerticalSplit(_) => "Vertical Split",
			Panes::HorizontalSplit(_) => "Horizontal Split",
			Panes::Tabbed(_) => "Tabs",
			Panes::Single(pane) => pane.name.as_str(),
		}
	}

	fn update(&mut self, ctx: &mut UpdatingContext<'_>) -> Action {
		use Panes::*;

		match self {
			VerticalSplit(list) | HorizontalSplit(list) | Tabbed(list) => {
				list.update(ctx)
			}
			Single(pane) => pane.update(ctx),
		}
	}
}

impl Default for Panes {
	fn default() -> Panes {
		Panes::Tabbed(PaneList::default())
	}
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct PaneList {
	pub panes: Vec<Panes>,
	pub focused: u32,
}

impl PaneList {
	fn update(&mut self, ctx: &mut UpdatingContext<'_>) -> Action {
		self.panes.iter_mut().map(|pane| pane.update(ctx)).sum()
	}
}
