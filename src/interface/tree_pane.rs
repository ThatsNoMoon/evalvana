use super::Pane;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TreePane {
	pub pane_statuses: PaneStatuses,
	pub evaluators: Evaluators,
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
