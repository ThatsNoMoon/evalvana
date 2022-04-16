// Copyright 2022 ThatsNoMoon
// Licensed under the Open Software License version 3.0

#![cfg_attr(windows, windows_subsystem = "windows")]

use std::{collections::HashMap, env, sync::Arc};

use anyhow::{anyhow, Context as _, Error};
use futures::executor::block_on;
use iced::{
	window::{self, Icon},
	Application, Color, Command, Container, Element, Length, Row, Settings,
	Space, Subscription,
};
use lazy_regex::{regex_captures, regex_is_match};

pub(crate) mod assets;
pub(crate) mod color;
pub(crate) mod config;
pub(crate) mod message;
pub(crate) mod model;
pub(crate) mod plugin;
pub(crate) mod style;

use crate::{
	assets::ICON64,
	config::Config,
	message::{InitMessage, Message},
	model::{PluginListing, Plugins, Tab, Tabs},
	plugin::{EnvironmentOutput, Plugin},
};

#[derive(Debug, Default)]
pub(crate) struct State {
	pub(crate) tabs: Tabs,
	pub(crate) plugins: Plugins,
	pub(crate) plugin_map: HashMap<Arc<str>, Plugin>,
	pub(crate) config: Config,
	running_envs: Vec<EnvironmentOutput>,
	loaded: bool,
}

impl Application for State {
	type Message = Message;

	type Executor = iced::executor::Default;

	type Flags = ();

	fn new(_flags: ()) -> (Self, Command<Message>) {
		let this = Self::default();

		let fut = async {
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
						errors.push(e.into());
						continue;
					}
				};

				let manifest = dir.join("manifest.json");

				match tokio::fs::read_to_string(&manifest)
					.await
					.with_context(|| {
						format!(
							"Failed to read manifest for plugin at {:?}",
							manifest
						)
					})
					.and_then(|manifest_text| {
						serde_json::from_str(&manifest_text).with_context(
							|| {
								format!(
									"Failed to parse manifest for plugin at {:?}",
									manifest
								)
							},
						)
					})
					.and_then(|mut plugin: Plugin| {
						if regex_is_match!(r"[^a-z0-9\-_]"i, &plugin.name) {
							return Err(anyhow!(
								r#"Invalid plugin name "{}""#,
								plugin.name
							));
						}

						#[cfg(windows)]
						match plugin.program.extension() {
							None => {
								plugin.program.set_extension("exe");
							}
							Some(_) => (),
						}

						plugin.program = which::which_in(
							&plugin.program,
							env::var_os("PATH"),
							&dir,
						)
						.with_context(|| {
							format!(
								"Failed to determine path \
									of program {:?} for plugin {}",
								plugin.program, plugin.name
							)
						})?;

						Ok(plugin)
					}) {
					Ok(plugin) => plugins.push(plugin),
					Err(e) => {
						errors.push(Message::Error(e.into()));
						continue;
					}
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
		};

		(
			this,
			Command::perform(fut, |result: Result<_, Error>| match result {
				Ok(msg) => msg,
				Err(e) => Message::Init(InitMessage::Error(e.into())),
			}),
		)
	}

	fn title(&self) -> String {
		"Evalvana".to_owned()
	}

	fn background_color(&self) -> Color {
		self.config.ui_colors.bg
	}

	fn update(&mut self, message: Self::Message) -> Command<Message> {
		match message {
			Message::OpenTab(plugin_name) => {
				let plugin = self
					.plugin_map
					.get_mut(&*plugin_name)
					.expect("Tried to open tab with non-existent plugin");

				let (env, output) = match plugin.open() {
					Ok(x) => x,
					Err(e) => {
						return Command::perform(async move { e }, Into::into)
					}
				};

				let tab = Tab::new(env, plugin.capabilities.clone());

				self.running_envs.push(output);

				self.tabs.push(tab);

				Command::none()
			}

			Message::SwitchTab(index) => {
				self.tabs.set_active(index);
				Command::none()
			}

			Message::CloseTab(index) => {
				let env = self.tabs.remove(index).env;
				self.running_envs.remove(index.0);

				Command::perform(
					async move { env.write().await.kill().await },
					Into::into,
				)
			}

			Message::NewContents(tab, cell, contents) => {
				self.tabs[tab].cells[cell].contents = contents;
				Command::none()
			}

			Message::Eval(tab_index, cell) => {
				let tab = &mut self.tabs[tab_index];
				let code = tab.cells[cell].contents.to_owned();
				let env = tab.env.clone();

				Command::perform(
					async move { env.write().await.eval_string(&code).await },
					move |res| match res {
						Ok(seq) => {
							Message::RequestInFlight(tab_index, cell, seq)
						}
						Err(e) => Message::Error(e.into()),
					},
				)
			}

			Message::RequestInFlight(tab, cell, seq) => {
				if let Some(t) = self.tabs.get_mut(tab) {
					t.request_in_flight(cell, seq);
				}

				Command::none()
			}

			Message::EvalComplete(env, seq, results) => {
				match self
					.tabs
					.iter_mut()
					.find(|tab| *block_on(tab.env.read()).id == *env)
				{
					Some(t) => {
						t.eval_complete(seq, results);
					}
					None => eprintln!(
						"Received eval results for an \
						environment with no tab: {}",
						env
					),
				}
				Command::none()
			}

			Message::NewCell(tab) => {
				self.tabs[tab].cells.new_cell();

				Command::none()
			}

			Message::Init(m) => match m {
				InitMessage::PluginListLoaded(plugins) => {
					self.plugins.list = plugins
						.iter()
						.map(|plugin| PluginListing::new(plugin.name.clone()))
						.collect();
					self.plugins
						.list
						.sort_unstable_by(|a, b| a.name.cmp(&b.name));
					self.plugin_map = plugins
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

			Message::Batch(msgs) => {
				Command::batch(msgs.into_iter().map(|msg| self.update(msg)))
			}

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

		let sidebar = self.plugins.view(&self.config);
		let sidebar = Container::new(sidebar)
			.style(style::container::secondary_bg(&self.config))
			.width(Length::Units(230))
			.height(Length::Fill)
			.padding([15, 0])
			.into();

		let content = self.tabs.view(&self.config);

		Row::with_children(vec![sidebar, content]).into()
	}
}

fn main() {
	env_logger::init();

	let icon = {
		let decoder = png::Decoder::new(ICON64);
		let mut reader = decoder.read_info().expect("Failed to read icon PNG");
		let mut buf = vec![0; reader.output_buffer_size()];
		let info = reader
			.next_frame(&mut buf)
			.expect("Failed to read icon PNG");
		buf.truncate(info.buffer_size());

		Icon::from_rgba(buf, info.width, info.height)
			.expect("Failed to create icon")
	};

	let settings = Settings {
		window: window::Settings {
			icon: Some(icon),
			min_size: Some((630, 400)),
			..window::Settings::default()
		},
		..Settings::default()
	};

	State::run(settings).expect("Failed to run app");
}
