// Copyright 2021 ThatsNoMoon
// Licensed under the Open Software License version 3.0

use crate::model::EnvironmentId;

#[derive(Debug, Clone)]
pub enum Message {
	OpenTab(EnvironmentId),
	SwitchTab(usize),
	CloseTab(usize),
	NewContents(usize, String),
}
