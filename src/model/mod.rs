// Copyright 2022 ThatsNoMoon
// Licensed under the Open Software License version 3.0

pub(crate) mod cell;

use std::{collections::HashMap, sync::Arc};

use evalvana_api::EvalResult;
use iced::{
	alignment, button, scrollable, Button, Column, Container, Element, Length,
	Row, Space, Text,
};
use tokio::sync::RwLock;

use self::cell::{Cell, Cells};
use crate::{
	assets::{
		font,
		icons::{self, CLOSE_TAB, EMPTY_TAB},
	},
	config::Config,
	message::Message,
	plugin::{Capabilities, Environment},
	style,
};

#[derive(Debug)]
pub(crate) struct Tab {
	pub(crate) env: Arc<RwLock<Environment>>,
	plugin_name: Arc<str>,
	tab_button_state: button::State,
	close_button_state: button::State,
	pub(crate) cells: Cells,
}

impl Tab {
	pub(crate) fn new(
		env: Environment,
		plugin_capabilities: Capabilities,
	) -> Self {
		let plugin_name = env.plugin_name.clone();

		let cells = if plugin_capabilities.multiple_cells {
			Cells::Multiple {
				cells: vec![Cell::default()],
				scrollable_state: scrollable::State::new(),
				new_cell_button_state: button::State::new(),
				in_flight_requests: HashMap::new(),
			}
		} else {
			Cells::Single(Cell::default())
		};

		Self {
			env: Arc::new(RwLock::new(env)),
			plugin_name,
			tab_button_state: button::State::new(),
			close_button_state: button::State::new(),
			cells,
		}
	}

	pub(crate) fn view<'s>(
		&'s mut self,
		config: &Config,
		is_active: bool,
		index: usize,
	) -> (Element<'s, Message>, Option<Element<'s, Message>>) {
		let tab_button = {
			let label = Text::new(&*self.plugin_name)
				.color(if is_active {
					config.ui_colors.text
				} else {
					config.ui_colors.unfocused_text
				})
				.size(config.text_settings.ui_font_size)
				.font(font::BODY);

			let label = Container::new(label).height(Length::Fill).center_y();

			let row = Row::with_children(vec![
				label.into(),
				Space::with_width(Length::Units(50)).into(),
			]);

			let button = Button::new(&mut self.tab_button_state, row)
				.height(Length::Fill)
				.style(style::button::tab_handle(config));

			if is_active {
				button
			} else {
				button.on_press(Message::SwitchTab(index))
			}
		};

		let close_button = {
			let icon = Text::new(CLOSE_TAB).font(icons::FONT).size(8);
			let icon = Container::new(icon).center_y().height(Length::Fill);

			Button::new(&mut self.close_button_state, icon)
				.height(Length::Fill)
				.style(style::button::tab_close(config, is_active))
				.on_press(Message::CloseTab(index))
		};

		let handle =
			Row::with_children(vec![tab_button.into(), close_button.into()])
				.into();

		let contents = if is_active {
			Some(self.cells.view(config, index))
		} else {
			None
		};

		(handle, contents)
	}

	pub(crate) fn request_in_flight(&mut self, cell: usize, seq: u32) {
		if let Cells::Multiple {
			cells,
			in_flight_requests,
			..
		} = &mut self.cells
		{
			if cell < cells.len() {
				in_flight_requests.insert(seq, cell);
			}
		}
	}

	pub(crate) fn eval_complete(&mut self, seq: u32, results: Vec<EvalResult>) {
		match &mut self.cells {
			Cells::Single(cell) => cell.results = results,
			Cells::Multiple {
				cells,
				in_flight_requests,
				..
			} => {
				if let Some(cell) =
					in_flight_requests.get(&seq).and_then(|&i| cells.get_mut(i))
				{
					cell.results = results;
				}
			}
		}
	}
}

#[derive(Debug, Default)]
pub(crate) struct Tabs {
	pub(crate) tabs: Vec<Tab>,
	active_tab: usize,
}

impl Tabs {
	pub(crate) fn push(&mut self, tab: Tab) {
		self.tabs.push(tab);
		self.active_tab = self.tabs.len() - 1;
	}

	pub(crate) fn remove(&mut self, index: usize) -> Tab {
		let tab = self.tabs.remove(index);
		self.active_tab = self.active_tab.saturating_sub(1);
		tab
	}

	pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut Tab> {
		self.tabs.iter_mut()
	}

	pub(crate) fn set_active(&mut self, index: usize) {
		if index >= self.tabs.len() {
			panic!(
				"Index {} out of bounds for tab list of length {}",
				index,
				self.tabs.len()
			);
		}
		self.active_tab = index;
	}

	pub(crate) fn view<'s>(
		&'s mut self,
		config: &Config,
	) -> Element<'s, Message> {
		if self.tabs.is_empty() {
			let placeholder_icon = Text::new(EMPTY_TAB)
				.font(icons::FONT)
				.size(256)
				.color(config.ui_colors.bg_icon);
			return Container::new(placeholder_icon)
				.center_x()
				.center_y()
				.width(Length::Fill)
				.height(Length::Fill)
				.style(style::container::ui_bg(config))
				.into();
		}

		let active_tab = self.active_tab;
		let mut content = None;
		let handles = self.tabs.iter_mut().enumerate().fold(
			Row::new()
				.height(iced::Length::Units(33))
				.push(Space::with_width(Length::Units(7))),
			|row, (i, tab)| {
				let (handle, contents) = tab.view(config, i == active_tab, i);

				if i == active_tab {
					content = contents;
				}

				row.push(handle).push(Space::with_width(Length::Units(7)))
			},
		);

		let handles = Container::new(handles)
			.style(style::container::tab_bg(config))
			.height(Length::Units(40))
			.width(Length::Fill)
			.align_y(alignment::Vertical::Bottom);

		Column::new()
			.push(handles)
			.push(content.expect(
				"Active tab index out of bounds, \
				or active tab produced no content",
			))
			.into()
	}
}

impl std::ops::Index<usize> for Tabs {
	type Output = Tab;

	fn index(&self, index: usize) -> &Self::Output {
		&self.tabs[index]
	}
}

impl std::ops::IndexMut<usize> for Tabs {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.tabs[index]
	}
}

#[derive(Debug, Clone)]
pub(crate) struct PluginListing {
	pub(crate) name: Arc<str>,
	button_state: button::State,
}

impl PluginListing {
	pub(crate) fn new(name: Arc<str>) -> Self {
		Self {
			name,
			button_state: button::State::new(),
		}
	}

	pub(crate) fn view<'s>(
		&'s mut self,
		config: &Config,
	) -> Element<'s, Message> {
		let text = Text::new(&*self.name)
			.size(config.text_settings.ui_font_size)
			.color(config.ui_colors.text);

		let inner = Container::new(text)
			.center_y()
			.height(Length::Fill)
			.width(Length::Fill);

		Button::new(&mut self.button_state, inner)
			.on_press(Message::OpenTab(self.name.clone()))
			.style(style::button::secondary(config))
			.padding(10)
			.height(Length::Units(70))
			.width(Length::Fill)
			.into()
	}

	pub(crate) fn view_list<'s>(
		info: &'s mut [Self],
		config: &Config,
	) -> Element<'s, Message> {
		let header = {
			let text = Text::new("Available REPLs")
				.size(config.text_settings.header_font_size)
				.color(config.ui_colors.accent)
				.font(font::BODY);

			Container::new(text)
				.center_x()
				.padding(5)
				.width(Length::Fill)
		};

		let column = Column::new().push(header);
		info.iter_mut()
			.fold(column, |column, info| {
				column
					.push(Space::with_height(Length::Units(15)))
					.push(info.view(config))
			})
			.into()
	}
}
