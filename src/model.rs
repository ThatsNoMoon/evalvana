// Copyright 2022 ThatsNoMoon
// Licensed under the Open Software License version 3.0

use std::sync::Arc;

use evalvana_api::EvalResult;
use evalvana_editor::{self as editor, TextInput};
use iced::{
	alignment, button, svg::Handle, Button, Column, Container, Element, Length,
	Row, Space, Svg, Text,
};
use tokio::sync::RwLock;

use crate::{
	assets::{font, EMPTY_TAB_ICON},
	config::Config,
	message::Message,
	plugin::Environment,
	style::{self, button::ButtonStyleSheet, text_input::TextInputStyleSheet},
};

#[derive(Debug)]
pub(crate) struct Tab {
	pub(crate) env: Arc<RwLock<Environment>>,
	plugin_name: Arc<str>,
	tab_button_state: button::State,
	close_button_state: button::State,
	input_state: editor::State,
	eval_button_state: button::State,
	pub(crate) contents: String,
	pub(crate) results: Vec<EvalResult>,
}

impl Tab {
	pub(crate) fn new(env: Environment) -> Self {
		let plugin_name = env.plugin_name.clone();
		Self {
			env: Arc::new(RwLock::new(env)),
			plugin_name,
			tab_button_state: button::State::new(),
			close_button_state: button::State::new(),
			input_state: editor::State::focused(),
			eval_button_state: button::State::new(),
			contents: String::new(),
			results: vec![],
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

			Button::new(&mut self.tab_button_state, row)
				.height(Length::Fill)
				.on_press(Message::SwitchTab(index))
				.style(if is_active {
					Box::new(style::tab::Active::from(config))
						as Box<dyn ButtonStyleSheet + 'static>
				} else {
					style::tab::Inactive::from(config).into()
				})
		};

		let close_button = {
			let icon = Text::new("x")
				.color(config.ui_colors.text)
				.size(config.text_settings.ui_font_size);
			let icon = Container::new(icon).center_y().height(Length::Fill);

			Button::new(&mut self.close_button_state, icon)
				.height(Length::Fill)
				.style(if is_active {
					Box::new(style::button::Secondary::from(config))
						as Box<dyn ButtonStyleSheet + 'static>
				} else {
					style::tab::Inactive::from(config).into()
				})
				.on_press(Message::CloseTab(index))
		};

		let handle =
			Row::with_children(vec![tab_button.into(), close_button.into()])
				.into();

		if is_active {
			let input = TextInput::new(
				&mut self.input_state,
				"",
				&self.contents,
				move |contents| Message::NewContents(index, contents),
			)
			.size(config.text_settings.editor_font_size)
			.style(Box::new(style::text_input::Editor::from(config))
				as Box<dyn TextInputStyleSheet + 'static>)
			.font(font::MONO);

			let input = Container::new(input)
				.style(style::container::UiBg::from(config))
				.width(Length::Fill)
				.height(Length::Fill);

			let divider =
				Container::new(Space::new(Length::Fill, Length::Units(3)))
					.style(style::container::SecondaryBg::from(config));

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

					let text = Text::new(msg)
						.size(config.text_settings.editor_font_size)
						.color(color)
						.font(font::MONO);

					Column::new()
						.push(text)
						.push(Space::with_height(Length::Units(10)))
						.into()
				})
				.collect();

			let results = Column::with_children(results);

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
					.style(style::button::Primary::from(config))
					.on_press(Message::Eval(index))
			};

			let contents = Column::new()
				.push(input)
				.push(divider)
				.push(results)
				.push(eval_button);

			let contents = Container::new(contents)
				.padding(20)
				.width(Length::Fill)
				.height(Length::Fill);

			(handle, Some(contents.into()))
		} else {
			(handle, None)
		}
	}
}

#[derive(Debug)]
pub(crate) struct Tabs {
	tabs: Vec<Tab>,
	active_tab: usize,
	placeholder_icon: Handle,
}

impl Default for Tabs {
	fn default() -> Self {
		Self {
			tabs: vec![],
			active_tab: 0,
			placeholder_icon: Handle::from_memory(EMPTY_TAB_ICON),
		}
	}
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
			let placeholder_icon = Svg::new(self.placeholder_icon.clone())
				.width(Length::Units(256))
				.height(Length::Units(256));
			return Container::new(placeholder_icon)
				.center_x()
				.center_y()
				.width(Length::Fill)
				.height(Length::Fill)
				.style(style::container::UiBg::from(config))
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
			.style(style::container::TabBg::from(config))
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
			.style(style::button::Secondary::from(config))
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
