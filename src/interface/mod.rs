mod pane;
pub use pane::Pane;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Interface {
	pub panes: Panes,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Panes {
	VerticalSplit(PaneList),
	HorizontalSplit(PaneList),
	Tabbed(PaneList),
	Single(Pane),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PaneList {
	pub panes: Vec<Panes>,
	pub focused: usize,
}

impl Default for Panes {
	fn default() -> Panes {
		Panes::Tabbed(PaneList::default())
	}
}