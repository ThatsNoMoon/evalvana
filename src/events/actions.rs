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
	None,
	Single(ActionData),
	Many(TinyVec<[ActionData; Action::INLINE_CAPACITY]>),
}

impl Action {
	pub const INLINE_CAPACITY: usize = 16;

	pub fn simplify(&mut self) {
		match self {
			Action::Single(ActionData::None) => *self = Action::None,
			_ => (),
		}
	}

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
		Action::None
	}
}

impl Add<Action> for Action {
	type Output = Self;
	fn add(mut self, mut other: Self) -> Self {
		self.simplify();
		other.simplify();

		let mut res = match self {
			Action::None => other,
			Action::Many(mut vec) => match other {
				Action::None => Action::Many(vec),
				Action::Many(mut other_vec) => {
					vec.append(&mut other_vec);
					Action::Many(vec)
				}
				Action::Single(data) => {
					vec.push(data);
					Action::Many(vec)
				}
			},
			Action::Single(data) => match other {
				Action::None => Action::Single(data),
				Action::Many(mut other_vec) => {
					other_vec.push(data);
					Action::Many(other_vec)
				}
				Action::Single(other_data) => Action::Many(tiny_vec![
					[ActionData; Action::INLINE_CAPACITY],
					data,
					other_data
				]),
			},
		};

		res.simplify();
		res
	}
}

impl Add<ActionData> for Action {
	type Output = Self;
	fn add(mut self, other: ActionData) -> Self {
		self.simplify();
		let mut res = match self {
			Action::None => Action::from(other),
			Action::Many(mut vec) => match other {
				ActionData::None => Action::Many(vec),
				data => {
					vec.push(data);
					Action::Many(vec)
				}
			},
			Action::Single(data) => match other {
				ActionData::None => Action::Single(data),
				other_data => Action::Many(tiny_vec![
					[ActionData; Action::INLINE_CAPACITY],
					data,
					other_data
				]),
			},
		};

		res.simplify();
		res
	}
}

impl From<ActionData> for Action {
	fn from(data: ActionData) -> Self {
		match data {
			ActionData::None => Self::None,
			data => Self::Single(data),
		}
	}
}

impl From<TinyVec<[ActionData; Action::INLINE_CAPACITY]>> for Action {
	fn from(data: TinyVec<[ActionData; Action::INLINE_CAPACITY]>) -> Self {
		Self::Many(data)
	}
}

impl Sum<Action> for Action {
	fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
		iter.fold(Action::None, |acc, action| acc + action)
	}
}

impl Sum<ActionData> for Action {
	fn sum<I: Iterator<Item = ActionData>>(iter: I) -> Self {
		iter.fold(Action::None, |acc, action| acc + action)
	}
}
