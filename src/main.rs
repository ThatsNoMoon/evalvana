#![allow(dead_code, unused_imports)]

pub mod app;
pub mod config;
pub mod input;
pub mod interface;
pub mod renderer;
pub mod repl;

fn main() {
	env_logger::init();

	let app = app::App::new();

	app.run();
}
