// Copyright 2021 ThatsNoMoon
// Licensed under the Open Software License version 3.0

use std::sync::Arc;

use crate::{
	assets::font,
	config::Config,
	message::Message,
	plugin::Environment,
	style::{self, button::ButtonStyleSheet, text_input::TextInputStyleSheet},
};
use evalvana_api::EvalResult;
use iced::{
	button, text_input, Button, Column, Container, Element, Length, Row, Space,
	Text, TextInput,
};
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct Tab {
	pub env: Arc<RwLock<Environment>>,
	plugin_name: Arc<str>,
	tab_button_state: button::State,
	close_button_state: button::State,
	input_state: text_input::State,
	eval_button_state: button::State,
	pub contents: String,
	pub results: Vec<EvalResult>,
}

impl Tab {
	pub fn new(env: Environment) -> Self {
		let plugin_name = env.plugin_name.clone();
		Self {
			env: Arc::new(RwLock::new(env)),
			plugin_name,
			tab_button_state: button::State::new(),
			close_button_state: button::State::new(),
			input_state: text_input::State::focused(),
			eval_button_state: button::State::new(),
			contents: String::new(),
			results: vec![],
		}
	}

	pub fn view<'s>(
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
				.width(Length::Fill);

			let space = Space::with_height(Length::Fill);

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
				.push(space)
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
pub struct Tabs {
	pub tabs: Vec<Tab>,
	pub active_tab: usize,
}

impl Tabs {
	pub fn view<'s>(&'s mut self, config: &Config) -> Element<'s, Message> {
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
			.align_y(iced::Align::End);

		Column::new().push(handles).push(content.unwrap()).into()
	}
}

#[derive(Debug, Clone)]
pub struct PluginListing {
	pub name: Arc<str>,
	button_state: button::State,
}

impl PluginListing {
	pub fn new(name: Arc<str>) -> Self {
		Self {
			name,
			button_state: button::State::new(),
		}
	}

	pub fn view<'s>(&'s mut self, config: &Config) -> Element<'s, Message> {
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

	pub fn view_list<'s>(
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
