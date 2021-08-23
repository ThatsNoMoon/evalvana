// Copyright 2021 ThatsNoMoon
// Licensed under the Open Software License version 3.0

use anyhow::Error;
use evalvana_api::EvalResult;
use std::sync::Arc;

use crate::plugin::Plugin;

#[derive(Debug, Clone)]
pub enum Message {
	Init(InitMessage),
	OpenTab(Arc<str>),
	SwitchTab(usize),
	CloseTab(usize),
	NewContents(usize, String),
	Error(Arc<Error>),
	Batch(Vec<Message>),
	Eval(usize),
	EvalComplete(String, u32, Vec<EvalResult>),
	Nothing,
}

#[derive(Debug, Clone)]
pub enum InitMessage {
	PluginListLoaded(Vec<Plugin>),
	Error(Arc<Error>),
}

impl From<Result<Message, Error>> for Message {
	fn from(res: Result<Message, Error>) -> Self {
		match res {
			Ok(m) => m,
			Err(e) => Message::Error(e.into()),
		}
	}
}

impl From<Result<(), Error>> for Message {
	fn from(res: Result<(), Error>) -> Self {
		match res {
			Ok(()) => Message::Nothing,
			Err(e) => Message::Error(e.into()),
		}
	}
}
