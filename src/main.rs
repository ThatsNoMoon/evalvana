#![allow(dead_code, unused_imports)]

pub mod app;
pub mod interface;
pub mod renderer;
pub mod repl;
pub mod config;
pub mod input;

fn main() {
	env_logger::init();

	let app = app::App::new();

	app.run();

}

