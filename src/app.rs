use crate::config::{Config, UiColors, EditorColors};
use crate::interface::Interface;
use crate::renderer::{
	Renderer,
	color::Color,
};

use winit::{
	event::{Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	window::{WindowBuilder, Window, Icon},
};
use image::{
	png::PngDecoder,
	ImageDecoder,
};

const ICON: &'static [u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/logo.png"));

pub struct App {
	window: Window,
	event_loop: EventLoop<()>,
	renderer: Renderer,
	interface: Interface,
	config: Config,
}

impl App {
	pub fn new() -> App {
		let config = Config {
			ui_colors: UiColors {
				bg: Color::from_rgb_u32(0x282C34),
				text: Color::from_rgb_u32(0xABB2BF),
				borders: Color::from_rgb_u32(0x4B5263),
			},
			editor_colors: EditorColors {
				bg: Color::from_rgb_u32(0x282C34),
				main: Color::from_rgb_u32(0xABB2BF),
				strings: Color::from_rgb_u32(0x98C379),
				numbers: Color::from_rgb_u32(0xD19A66),
				operators: Color::from_rgb_u32(0xC678DD),
				keywords: Color::from_rgb_u32(0xE06C75),
				variables: Color::from_rgb_u32(0xE5C07B),
				parameters: Color::from_rgb_u32(0xE5C07B),
				constants: Color::from_rgb_u32(0x56B6C2),
				types: Color::from_rgb_u32(0x61AFEF),
				functions: Color::from_rgb_u32(0xABB2BF),
			}
		};


		let event_loop = EventLoop::new();

		let window = WindowBuilder::new().build(&event_loop).unwrap();

		{
			let logo_decoder = PngDecoder::new(ICON).unwrap();
			let (logo_w, logo_h) = logo_decoder.dimensions();
			let mut logo_pixels = vec![0; logo_decoder.total_bytes() as usize];
			logo_decoder.read_image(logo_pixels.as_mut_slice()).unwrap();

			let icon = Icon::from_rgba(logo_pixels, logo_w, logo_h).unwrap();

			window.set_window_icon(Some(icon));
		}

		window.set_title("Evalvana");

		let renderer = Renderer::new(&window);

		let interface = Interface::default();

		App {
			window,
			event_loop,
			renderer,
			interface,
			config,
		}
	}

	pub fn run(self) {
		let App { window, event_loop, mut renderer, mut interface, config, .. } = self;
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
					renderer.redraw(&window, &config, &mut interface);
				},

				Event::WindowEvent {
					event: WindowEvent::CloseRequested,
					window_id,
				} if window_id == window.id() => *control_flow = ControlFlow::Exit,
				_ => (),
			}
		});
	}
}
