#![allow(dead_code, unused_imports)]

use winit::{
	event::{Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	window::WindowBuilder,
};

use cgmath::{Vector2, vec2};

mod config;
use config::{Config, UiColors, EditorColors};

mod renderer;
use renderer::{
	Renderer,
	color::Color,
};

mod repl;

mod input;

pub mod app;

fn main() {
	env_logger::init();


	event_loop.run(move |event, _, control_flow| {
		*control_flow = ControlFlow::Poll;

		match event {
			Event::MainEventsCleared => window.request_redraw(),

			Event::WindowEvent {
				event: WindowEvent::Resized(size),
				..
			} => {
				renderer.resize(size);
			},

			Event::RedrawRequested(_) => {
				renderer.redraw(&config);
			},

			Event::WindowEvent {
				event: WindowEvent::CloseRequested,
				window_id,
			} if window_id == window.id() => *control_flow = ControlFlow::Exit,
			_ => (),
		}
	});
}

