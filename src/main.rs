#![allow(dead_code, unused_imports)]

pub mod app;
pub mod config;
pub mod events;
pub mod geometry;
pub mod icons;
pub mod interface;
pub mod rendering;
pub mod repl;
pub mod util;

fn main() {
	env_logger::init();

	let app = app::App::default();

	app.run();
}
