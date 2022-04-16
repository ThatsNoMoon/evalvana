// Copyright 2022 ThatsNoMoon
// Licensed under the Open Software License version 3.0

use std::{
	collections::HashMap,
	ops::{Index, IndexMut},
};

use evalvana_api::EvalResult;
use evalvana_editor::{self as editor, TextInput};
use iced::{
	button, scrollable, Button, Column, Container, Element, Length, Row, Rule,
	Scrollable, Space, Text,
};

use crate::{
	assets::{
		font,
		icons::{self, NEW_CELL},
	},
	config::Config,
	message::Message,
	style::{self, text_input::TextInputStyleSheet},
};

#[derive(Debug)]
pub(crate) struct Cell {
	input_state: editor::State,
	eval_button_state: button::State,
	pub(crate) contents: String,
	pub(crate) results: Vec<EvalResult>,
}

impl Default for Cell {
	fn default() -> Self {
		Self {
			input_state: editor::State::focused(),
			eval_button_state: button::State::new(),
			contents: String::new(),
			results: vec![],
		}
	}
}

impl Cell {
	pub(super) fn view<'s>(
		&'s mut self,
		config: &Config,
		tab_index: usize,
		index: usize,
	) -> Element<'s, Message> {
		let input = TextInput::new(
			&mut self.input_state,
			"",
			&self.contents,
			move |contents| Message::NewContents(tab_index, index, contents),
		)
		.size(config.text_settings.editor_font_size)
		.style(Box::new(style::text_input::Editor::from(config))
			as Box<dyn TextInputStyleSheet + 'static>)
		.font(font::MONO);

		let input = Container::new(input)
			.style(style::container::ui_bg(config))
			.width(Length::Fill)
			.height(Length::Fill);

		let divider =
			Rule::horizontal(21).style(style::rule::cell_divider(config, 1));

		let results = self
			.results
			.iter()
			.map(|result| {
				let (color, msg) = match result {
					EvalResult::Success(msg) => {
						(config.editor_colors.success, &*msg.text)
					}
					EvalResult::Warning(msg) => {
						(config.editor_colors.warnings, &*msg.text)
					}
					EvalResult::Error(msg) => {
						(config.editor_colors.errors, &*msg.text)
					}
				};

				Text::new(msg)
					.size(config.text_settings.editor_font_size)
					.color(color)
					.font(font::MONO)
					.into()
			})
			.collect();

		let results = Column::with_children(results).spacing(10);

		let eval_button = {
			let text = Text::new("Eval")
				.color(config.ui_colors.text)
				.size(config.text_settings.ui_font_size);

			let text = Container::new(text).padding(10);

			let contents = Row::new()
				.push(Space::with_width(Length::Units(10)))
				.push(text)
				.push(Space::with_width(Length::Units(10)));

			Button::new(&mut self.eval_button_state, contents)
				.style(style::button::primary(config))
				.on_press(Message::Eval(tab_index, index))
		};

		Column::new()
			.push(input)
			.push(divider)
			.push(results)
			.push(Space::new(Length::Shrink, Length::Units(10)))
			.push(eval_button)
			.into()
	}
}

#[derive(Debug)]
pub(crate) enum Cells {
	Single(Cell),
	Multiple {
		cells: Vec<Cell>,
		scrollable_state: scrollable::State,
		new_cell_button_state: button::State,
		in_flight_requests: HashMap<u32, usize>,
	},
}

impl Cells {
	pub(super) fn view<'s>(
		&'s mut self,
		config: &Config,
		tab_index: usize,
	) -> Element<'s, Message> {
		match self {
			Cells::Single(cell) => {
				let cell_contents = cell.view(config, tab_index, 0);

				let contents = Container::new(cell_contents)
					.padding(20)
					.width(Length::Fill)
					.height(Length::Fill);

				contents.into()
			}

			Cells::Multiple {
				cells,
				scrollable_state,
				new_cell_button_state,
				..
			} => {
				let scrollable = cells
					.iter_mut()
					.enumerate()
					.map(|(cell_index, cell)| {
						let contents = cell.view(config, tab_index, cell_index);
						let contents = Container::new(contents)
							.padding(20)
							.width(Length::Fill)
							.height(Length::Units(300));

						let divider = Rule::horizontal(3)
							.style(style::rule::cell_divider(config, 2));

						Column::new().push(contents).push(divider).into()
					})
					.fold(
						Scrollable::new(scrollable_state),
						|scrollable, cell: Element<_>| scrollable.push(cell),
					);

				let new_cell = {
					let contents = Container::new(
						Text::new(NEW_CELL).font(icons::FONT).size(35),
					)
					.center_x()
					.center_y()
					.width(Length::Fill)
					.height(Length::Fill);
					Button::new(new_cell_button_state, contents)
						.style(style::button::new_cell(config))
						.on_press(Message::NewCell(tab_index))
						.width(Length::Fill)
						.height(Length::Units(200))
				};

				let scrollable = scrollable.push(new_cell);

				let contents = Container::new(scrollable)
					.style(style::container::editor_bg(config))
					.height(Length::Fill);

				contents.into()
			}
		}
	}

	pub(crate) fn new_cell(&mut self) {
		match self {
			Cells::Single(_) => panic!(
				"Attempted to create a new cell \
    	in a tab without multiple cells"
			),
			Cells::Multiple { cells, .. } => {
				cells.push(Cell::default());
			}
		}
	}
}

impl Index<usize> for Cells {
	type Output = Cell;

	fn index(&self, index: usize) -> &Self::Output {
		match self {
			Self::Single(cell) => {
				if index == 0 {
					cell
				} else {
					panic!(
						"Non-zero index used to get cell \
        				for tab without multiple cells"
					)
				}
			}
			Cells::Multiple { cells, .. } => &cells[index],
		}
	}
}

impl IndexMut<usize> for Cells {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		match self {
			Self::Single(cell) => {
				if index == 0 {
					cell
				} else {
					panic!(
						"Non-zero index used to get cell \
        				for tab without multiple cells"
					)
				}
			}
			Cells::Multiple { cells, .. } => &mut cells[index],
		}
	}
}
