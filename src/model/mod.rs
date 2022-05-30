// Copyright 2022 ThatsNoMoon
// Licensed under the Open Software License version 3.0

pub(crate) mod cell;

use std::{collections::HashMap, fmt, sync::Arc};

use evalvana_api::EvalResult;
use iced::{
	alignment, button, scrollable, Button, Column, Container, Element, Length,
	Row, Rule, Scrollable, Space, Text,
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TabIndex(pub(crate) usize);

impl fmt::Display for TabIndex {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Display::fmt(&self.0, f)
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CellIndex(pub(crate) usize);

impl fmt::Display for CellIndex {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Display::fmt(&self.0, f)
	}
}

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
		index: TabIndex,
	) -> (Element<'s, Message>, Option<Element<'s, Message>>) {
		let text_size = config.text_settings.ui_font_size;
		let tab_button = {
			let label = Text::new(&*self.plugin_name)
				.color(if is_active {
					config.ui_colors.text
				} else {
					config.ui_colors.unfocused_text
				})
				.size(text_size)
				.font(font::BODY);

			let label = Container::new(label)
				.height(Length::Fill)
				.padding([0, text_size / 2])
				.center_y();

			let row = Row::with_children(vec![
				label.into(),
				Space::with_width(Length::Units(text_size * 3)).into(),
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
			let icon =
				Text::new(CLOSE_TAB).font(icons::FONT).size(text_size / 2);
			let icon = Container::new(icon).center_y().height(Length::Fill);

			Button::new(&mut self.close_button_state, icon)
				.height(Length::Fill)
				.style(style::button::tab_close(config, is_active))
				.padding([0, text_size * 3 / 4])
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

	pub(crate) fn request_in_flight(&mut self, cell: CellIndex, seq: u32) {
		if let Cells::Multiple {
			cells,
			in_flight_requests,
			..
		} = &mut self.cells
		{
			if cell < CellIndex(cells.len()) {
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
				if let Some(cell) = in_flight_requests
					.get(&seq)
					.and_then(|&CellIndex(i)| cells.get_mut(i))
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
	active_tab: TabIndex,
}

impl Tabs {
	pub(crate) fn push(&mut self, tab: Tab) {
		self.tabs.push(tab);
		self.active_tab = TabIndex(self.tabs.len() - 1);
	}

	pub(crate) fn remove(&mut self, index: TabIndex) -> Tab {
		let tab = self.tabs.remove(index.0);
		if index <= self.active_tab {
			self.active_tab.0 = self.active_tab.0.saturating_sub(1);
		}
		tab
	}

	pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut Tab> {
		self.tabs.iter_mut()
	}

	pub(crate) fn get_mut(&mut self, index: TabIndex) -> Option<&mut Tab> {
		self.tabs.get_mut(index.0)
	}

	pub(crate) fn set_active(&mut self, index: TabIndex) {
		if index >= TabIndex(self.tabs.len()) {
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
				.size(250)
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
		let last_tab = self.tabs.len() - 1;
		let handles = self.tabs.iter_mut().enumerate().fold(
			Row::new().height(Length::Fill),
			|row, (i, tab)| {
				let i = TabIndex(i);
				let (handle, contents) = tab.view(config, i == active_tab, i);

				if i == active_tab {
					content = contents;
				}

				let row = row.push(handle);

				let divider_width = if i != active_tab
					&& TabIndex(i.0 + 1) != active_tab
					&& i.0 != last_tab
				{
					1
				} else {
					0
				};

				row.push(
					Rule::vertical(1)
						.style(style::rule::tab_divider(config, divider_width)),
				)
			},
		);

		let handles = Container::new(handles)
			.style(style::container::secondary_bg(config))
			.height(Length::Units(config.text_settings.ui_font_size * 3))
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

impl std::ops::Index<TabIndex> for Tabs {
	type Output = Tab;

	fn index(&self, index: TabIndex) -> &Self::Output {
		&self.tabs[index.0]
	}
}

impl std::ops::IndexMut<TabIndex> for Tabs {
	fn index_mut(&mut self, index: TabIndex) -> &mut Self::Output {
		&mut self.tabs[index.0]
	}
}

#[derive(Debug, Default)]
pub(crate) struct Plugins {
	pub(crate) list: Vec<PluginListing>,
	scrollable_state: scrollable::State,
}

impl Plugins {
	pub(crate) fn view<'s>(
		&'s mut self,
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

		let list = Scrollable::new(&mut self.scrollable_state)
			.push(header)
			.push(Space::with_height(Length::Units(15)));
		self.list
			.iter_mut()
			.fold(list, |list, info| list.push(info.view(config)))
			.into()
	}
}

#[derive(Debug)]
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
			.padding(10)
			.height(Length::Fill)
			.width(Length::Fill);

		Button::new(&mut self.button_state, inner)
			.on_press(Message::OpenTab(self.name.clone()))
			.style(style::button::primary(config))
			.height(Length::Units(70))
			.width(Length::Fill)
			.into()
	}
}
