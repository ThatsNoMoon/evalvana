// Copyright 2020 Benjamin Scherer
// Licensed under the Open Software License version 3.0

pub mod actions;

pub mod commands;
use commands::Command;

use winit::event::Event as WinitEvent;

#[derive(Debug)]
pub enum Event<'a> {
	Startup,
	WinitEvent(WinitEvent<'a, ()>),
	Command(Command),
}
