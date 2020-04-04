mod pane;
pub use pane::Pane;
mod tree_pane;
pub use tree_pane::{
	Evaluator, Evaluators, PaneStatus, PaneStatuses, TreePane,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Interface {
	pub panes: Panes,
	pub tree_pane: TreePane,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PaneList {
	pub panes: Vec<Panes>,
	pub focused: u32,
}

impl Default for Panes {
	fn default() -> Panes {
		Panes::Tabbed(PaneList::default())
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
	Dark,
	Light,
}

impl Default for Theme {
	fn default() -> Theme {
		Theme::Dark
	}
}
