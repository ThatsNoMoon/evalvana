// Copyright 2021 ThatsNoMoon
// Licensed under the Open Software License version 3.0

#![cfg_attr(windows, windows_subsystem = "windows")]

use std::{collections::HashMap, sync::Arc};

use anyhow::{anyhow, Context as _, Error};
use futures::{executor::block_on, FutureExt};
use iced::{
	Application, Clipboard, Color, Command, Container, Element, Length, Row,
	Settings, Space, Subscription,
};
use lazy_regex::{regex_captures, regex_is_match};

pub mod assets;
pub mod color;
pub mod config;
pub mod message;
pub mod model;
pub mod plugin;
pub mod style;

use crate::{
	config::Config,
	message::{InitMessage, Message},
	model::{PluginListing, Tab, Tabs},
	plugin::{EnvironmentOutput, Plugin},
};

#[derive(Debug, Default)]
pub struct State {
	pub tabs: Tabs,
	pub plugin_listings: Vec<PluginListing>,
	pub plugins: HashMap<Arc<str>, Plugin>,
	pub config: Config,
	running_envs: Vec<EnvironmentOutput>,
	loaded: bool,
}

impl Application for State {
	type Message = Message;

	type Executor = iced_futures::executor::Tokio;

	type Flags = ();

	fn new(_flags: ()) -> (Self, Command<Message>) {
		let this = Self::default();

		let cmd = async {
			let data_dir = dirs::data_dir()
				.context("Failed to get data dir")?
				.join("Evalvana");
			tokio::fs::create_dir_all(&data_dir)
				.await
				.context("Failed to create data dir")?;
			let plugin_dir = data_dir.join("plugins");
			tokio::fs::create_dir_all(&plugin_dir)
				.await
				.context("Failed to create plugin dir")?;

			let mut entries = tokio::fs::read_dir(&plugin_dir)
				.await
				.context("Failed to read plugin dir")?;

			let mut plugins = vec![];
			let mut errors = vec![];

			while let Some(entry) = entries
				.next_entry()
				.await
				.context("Failed to get plugin dir entry")?
			{
				let dir = match entry
					.file_type()
					.await
					.context("Failed to get plugin dir entry file type")
				{
					Ok(t) => match t.is_dir() {
						false => continue,
						true => entry.path(),
					},

					Err(e) => {
						errors.push(Message::Error(e.into()));
						continue;
					}
				};

				let manifest = dir.join("manifest.json");

				match tokio::fs::read_to_string(&manifest)
					.await
					.with_context(|| {
						format!(
							"Failed to read manifest for plugin {}",
							entry.file_name().to_string_lossy()
						)
					})
					.and_then(|manifest_text| {
						serde_json::from_str(&manifest_text).with_context(
							|| {
								format!(
									"Failed to parse manifest for plugin {}",
									entry.file_name().to_string_lossy()
								)
							},
						)
					})
					.and_then(|mut plugin: Plugin| {
						if regex_is_match!(r"[^a-z0-9\-_]"i, &plugin.name) {
							return Err(anyhow!(
								"Invalid plugin name: {}",
								plugin.name
							));
						}

						if plugin.program.is_relative() {
							plugin.program = dir.join(&plugin.program);
						}

						plugins.push(plugin);
						Ok(())
					}) {
					Ok(()) => (),
					Err(e) => errors.push(Message::Error(e.into())),
				}
			}

			let msg = match errors.len() {
				0 => Message::Init(InitMessage::PluginListLoaded(plugins)),
				_ => {
					errors.push(Message::Init(InitMessage::PluginListLoaded(
						plugins,
					)));
					Message::Batch(errors)
				}
			};

			Ok(msg)
		}
		.map(|result: Result<Message, Error>| match result {
			Ok(msg) => msg,
			Err(e) => Message::Init(InitMessage::Error(e.into())),
		})
		.into();

		(this, cmd)
	}

	fn title(&self) -> String {
		"Evalvana".to_owned()
	}

	fn background_color(&self) -> Color {
		self.config.ui_colors.bg
	}

	fn update(
		&mut self,
		message: Self::Message,
		clipboard: &mut Clipboard,
	) -> Command<Message> {
		match message {
			Message::OpenTab(plugin_name) => {
				let plugin = self
					.plugins
					.get_mut(&*plugin_name)
					.expect("Tried to open tab with non-existent plugin");

				let (env, output) =
					plugin.open().expect("Failed to start plugin environment");

				let tab = Tab::new(env);

				self.running_envs.push(output);

				self.tabs.push(tab);

				Command::none()
			}

			Message::SwitchTab(index) => {
				self.tabs.set_active(index);
				Command::none()
			}

			Message::CloseTab(index) => {
				self.tabs.remove(index);
				Command::none()
			}

			Message::NewContents(tab, contents) => {
				self.tabs[tab].contents = contents;
				Command::none()
			}

			Message::Eval(tab) => {
				let tab = &mut self.tabs[tab];
				let code = tab.contents.to_owned();
				let env = tab.env.clone();

				async move { env.write().await.eval_string(&code).await.into() }
					.into()
			}

			Message::EvalComplete(env, _seq, results) => {
				match self.tabs.iter_mut().find(|tab| *block_on(tab.env.read()).id == *env) {
					Some(t) => t.results = results,
					None => eprintln!("Received eval results for an environment with no tab: {}", env),
				}
				Command::none()
			}

			Message::Init(m) => match m {
				InitMessage::PluginListLoaded(plugins) => {
					self.plugin_listings = plugins
						.iter()
						.map(|plugin| PluginListing::new(plugin.name.clone()))
						.collect();
					self.plugin_listings
						.sort_unstable_by(|a, b| a.name.cmp(&b.name));
					self.plugins = plugins
						.into_iter()
						.map(|plugin| (plugin.name.clone(), plugin))
						.collect();
					self.loaded = true;
					Command::none()
				}

				InitMessage::Error(e) => {
					eprintln!("Error: {:?}", e);
					Command::none()
				}
			},

			Message::Error(e) => {
				eprintln!("Error: {:?}", e);
				Command::none()
			}

			Message::Batch(msgs) => Command::batch(
				msgs.into_iter()
					.map(|msg| self.update(msg, &mut *clipboard)),
			),

			Message::Nothing => Command::none(),
		}
	}

	fn subscription(&self) -> Subscription<Self::Message> {
		Subscription::batch(
			self.running_envs
				.iter()
				.map(|env| Subscription::from_recipe(env.take())),
		)
		.map(|result| {
			let response = result?;
			let results = Result::from(response.data)?;
			let resp_id = response
				.rpc
				.id
				.context("Eval RPC response contained no ID")?;
			let (_, env_id, seq) =
				regex_captures!(r"^([^/]+/[^/]+)/([^/]+)$", &resp_id)
					.with_context(|| {
						format!("Invalid RPC response ID: {}", resp_id)
					})?;
			let seq = seq.parse().with_context(|| {
				format!("Invalid RPC response seq: {}", seq)
			})?;

			Ok(Message::EvalComplete(env_id.to_owned(), seq, results))
		})
		.map(|result| result.into())
	}

	fn view(&mut self) -> Element<'_, Self::Message> {
		if !self.loaded {
			return Space::new(Length::Fill, Length::Fill).into();
		}

		let sidebar =
			PluginListing::view_list(&mut self.plugin_listings, &self.config);
		let sidebar = Container::new(sidebar)
			.style(style::container::SecondaryBg::from(&self.config))
			.width(Length::Units(230))
			.height(Length::Fill)
			.padding(15)
			.into();

		let content = self.tabs.view(&self.config);

		Row::with_children(vec![sidebar, content]).into()
	}
}

fn main() {
	env_logger::init();

	State::run(Settings::default()).expect("Failed to run app");
}
