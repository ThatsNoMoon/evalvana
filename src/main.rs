// Copyright 2020 Benjamin Scherer
// Licensed under the Open Software License version 3.0

use iced::{
	Color, Column, Container, Element, Length, Row, Sandbox, Settings, Space,
};

pub mod color;
pub mod config;
pub mod message;
pub mod model;
pub mod style;

use crate::{
	config::Config,
	message::Message,
	model::{Environment, EnvironmentInfo, Tab, Tabs},
};
use model::EnvironmentId;

#[derive(Debug)]
pub struct State {
	pub tabs: Option<Tabs>,
	pub envs: Vec<EnvironmentInfo>,
	pub config: Config,
}

impl Sandbox for State {
	type Message = Message;

	fn new() -> Self {
		Self {
			tabs: None,
			envs: vec![
				EnvironmentInfo::new(EnvironmentId::new("Rust".to_owned())),
				EnvironmentInfo::new(EnvironmentId::new("Lua".to_owned())),
				EnvironmentInfo::new(EnvironmentId::new(
					"TypeScript".to_owned(),
				)),
			],
			config: Config::default(),
		}
	}

	fn title(&self) -> String {
		"Evalvana".to_owned()
	}

	fn background_color(&self) -> Color {
		self.config.ui_colors.bg
	}

	fn update(&mut self, message: Self::Message) {
		match message {
			Message::OpenTab(id) => {
				let env = Environment::new(id);
				let tab = Tab::new(env);

				match &mut self.tabs {
					Some(tabs) => {
						tabs.tabs.push(tab);
						tabs.active_tab = tabs.tabs.len() - 1;
					}
					None => {
						self.tabs = Some(Tabs {
							tabs: vec![tab],
							active_tab: 0,
						})
					}
				}
			}

			Message::SwitchTab(index) => {
				self.tabs.as_mut().unwrap().active_tab = index;
			}

			Message::CloseTab(index) => {
				let tabs = self.tabs.as_mut().unwrap();
				tabs.tabs.remove(index);

				if tabs.tabs.is_empty() {
					self.tabs = None;
				} else {
					tabs.active_tab =
						tabs.active_tab.checked_sub(1).unwrap_or(0);
				}
			}
		}
	}

	fn view(&mut self) -> Element<'_, Self::Message> {
		let sidebar = EnvironmentInfo::view_list(&mut self.envs, &self.config);
		let sidebar = Container::new(sidebar)
			.style(style::container::SecondaryBg::from(&self.config))
			.width(Length::Units(230))
			.height(Length::Fill)
			.padding(15)
			.into();

		let content: Element<_> = match &mut self.tabs {
			Some(tabs) => tabs.view(&self.config).into(),
			None => {
				let handles_placeholder =
					Space::new(Length::Fill, Length::Fill);
				let handles_placeholder = Container::new(handles_placeholder)
					.width(Length::Fill)
					.height(Length::Units(40))
					.style(style::container::TabBg::from(&self.config));

				let content_placeholder =
					Space::new(Length::Fill, Length::Fill);
				let content_placeholder = Container::new(content_placeholder)
					.style(style::container::TabBg::from(&self.config));

				Column::with_children(vec![
					handles_placeholder.into(),
					content_placeholder.into(),
				])
				.into()
			}
		};

		Row::with_children(vec![sidebar, content]).into()
	}
}

fn main() {
	env_logger::init();

	State::run(Settings::default()).expect("Failed to run app");
}
