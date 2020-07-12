use std::{iter::Sum, ops::Add};

use tinyvec::{tiny_vec, TinyVec};

#[derive(Debug, PartialEq)]
pub enum ActionData {
	None,
	RequestRedraw,
}

impl Default for ActionData {
	fn default() -> Self {
		ActionData::None
	}
}

#[derive(Debug, PartialEq)]
pub enum Action {
	Single(ActionData),
	Many(TinyVec<[ActionData; Action::INLINE_CAPACITY]>),
}

impl Action {
	pub const INLINE_CAPACITY: usize = 16;

	pub const fn none() -> Self {
		Self::Single(ActionData::None)
	}

	pub fn simplify(&mut self) {}

	pub fn requests_redraw(&self) -> bool {
		match self {
			Action::Single(ActionData::RequestRedraw) => true,
			Action::Many(vec) => vec
				.iter()
				.any(|action| matches!(action, ActionData::RequestRedraw)),
			_ => false,
		}
	}
}

impl Default for Action {
	fn default() -> Action {
		Self::none()
	}
}

impl Add<Action> for Action {
	type Output = Self;
	fn add(mut self, mut other: Self) -> Self {
		self.simplify();
		other.simplify();

		let mut res = match self {
			Action::Single(ActionData::None) => other,
			Action::Many(mut vec) => match other {
				Action::Single(ActionData::None) => Action::Many(vec),
				Action::Single(data) => {
					vec.push(data);
					Action::Many(vec)
				}
				Action::Many(mut other_vec) => {
					vec.append(&mut other_vec);
					Action::Many(vec)
				}
			},
			Action::Single(data) => match other {
				Action::Single(ActionData::None) => Action::Single(data),
				Action::Single(other_data) => {
					Action::Many(tiny_vec![data, other_data])
				}
				Action::Many(mut other_vec) => {
					other_vec.push(data);
					Action::Many(other_vec)
				}
			},
		};

		res.simplify();
		res
	}
}

impl Add<ActionData> for Action {
	type Output = Self;
	fn add(self, data: ActionData) -> Self {
		self + Action::Single(data)
	}
}

impl From<ActionData> for Action {
	fn from(data: ActionData) -> Self {
		Self::Single(data)
	}
}

impl From<TinyVec<[ActionData; Action::INLINE_CAPACITY]>> for Action {
	fn from(data: TinyVec<[ActionData; Action::INLINE_CAPACITY]>) -> Self {
		Self::Many(data)
	}
}

impl Sum<Action> for Action {
	fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
		iter.fold(Action::none(), |acc, action| acc + action)
	}
}

impl Sum<ActionData> for Action {
	fn sum<I: Iterator<Item = ActionData>>(iter: I) -> Self {
		iter.fold(Action::none(), |acc, action| acc + action)
	}
}
