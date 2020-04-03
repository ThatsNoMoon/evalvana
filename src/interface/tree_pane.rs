use super::Pane;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TreePane {
	pub pane_statuses: PaneStatuses,
	pub evaluators: Evaluators,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaneStatuses {
	VerticalSplit(PaneStatusList),
	HorizontalSplit(PaneStatusList),
	Tabbed(PaneStatusList),
	Single(PaneStatus),
}

impl Default for PaneStatuses {
	fn default() -> PaneStatuses {
		PaneStatuses::Tabbed(PaneStatusList::default())
	}
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PaneStatusList {
	pub pane_statuses: Vec<PaneStatuses>,
	pub focused: Option<usize>,
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
