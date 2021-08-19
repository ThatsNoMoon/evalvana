// Copyright 2021 ThatsNoMoon
// Licensed under the Open Software License version 3.0

use crate::{assets::font, config::Config, message::Message, style::{self, button::ButtonStyleSheet, text_input::TextInputStyleSheet}};
use iced::{Button, Column, Container, Element, Length, Row, Space, Text, TextInput, button, text_input};

#[derive(Debug)]
pub struct Tab {
	pub env: Environment,
	tab_button_state: button::State,
	close_button_state: button::State,
	input_state: text_input::State,
	pub contents: String,
}

impl Tab {
	pub fn new(env: Environment) -> Self {
		Self {
			env,
			tab_button_state: button::State::new(),
			close_button_state: button::State::new(),
			input_state: text_input::State::focused(),
			contents: String::new(),
		}
	}

	pub fn view<'s>(
		&'s mut self,
		config: &Config,
		is_active: bool,
		index: usize,
	) -> (Element<'s, Message>, Option<Element<'s, Message>>) {
		let tab_button = {
			let label = Text::new(self.env.id.name.clone())
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
			let contents = Container::new(input)
				.padding(50)
				.style(style::container::UiBg::from(config))
				.width(Length::Fill)
				.height(Length::Fill)
				.into();

			(handle, Some(contents))
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
pub struct EnvironmentId {
	pub name: String,
}

impl EnvironmentId {
	pub fn new(name: String) -> Self {
		Self { name }
	}
}

#[derive(Debug)]
pub struct Environment {
	pub id: EnvironmentId,
}

impl Environment {
	pub fn new(id: EnvironmentId) -> Self {
		Self { id }
	}
}

#[derive(Debug, Clone)]
pub struct EnvironmentInfo {
	pub id: EnvironmentId,
	button_state: button::State,
}

impl EnvironmentInfo {
	pub fn new(id: EnvironmentId) -> Self {
		Self {
			id,
			button_state: button::State::new(),
		}
	}

	pub fn view<'s>(&'s mut self, config: &Config) -> Element<'s, Message> {
		let text = Text::new(&self.id.name)
			.size(config.text_settings.ui_font_size)
			.color(config.ui_colors.text);

		let inner = Container::new(text)
			.center_y()
			.height(Length::Fill)
			.width(Length::Fill);

		Button::new(&mut self.button_state, inner)
			.on_press(Message::OpenTab(self.id.clone()))
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
