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
