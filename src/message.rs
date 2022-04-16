// Copyright 2022 ThatsNoMoon
// Licensed under the Open Software License version 3.0

use std::sync::Arc;

use anyhow::Error;
use evalvana_api::EvalResult;

use crate::{
	model::{CellIndex, TabIndex},
	plugin::Plugin,
};

#[derive(Debug, Clone)]
pub(crate) enum Message {
	Init(InitMessage),
	OpenTab(Arc<str>),
	SwitchTab(TabIndex),
	CloseTab(TabIndex),
	NewContents(TabIndex, CellIndex, String),
	Error(Arc<Error>),
	Batch(Vec<Message>),
	Eval(TabIndex, CellIndex),
	RequestInFlight(TabIndex, CellIndex, u32),
	EvalComplete(String, u32, Vec<EvalResult>),
	NewCell(TabIndex),
	Nothing,
}

#[derive(Debug, Clone)]
pub(crate) enum InitMessage {
	PluginListLoaded(Vec<Plugin>),
	Error(Arc<Error>),
}

impl From<Result<Message, Error>> for Message {
	fn from(res: Result<Message, Error>) -> Self {
		match res {
			Ok(m) => m,
			Err(e) => e.into(),
		}
	}
}

impl From<Error> for Message {
	fn from(e: Error) -> Self {
		Message::Error(e.into())
	}
}

impl From<Result<(), Error>> for Message {
	fn from(res: Result<(), Error>) -> Self {
		match res {
			Ok(()) => Message::Nothing,
			Err(e) => e.into(),
		}
	}
}
