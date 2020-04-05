use crate::config::{Config, EditorColors, UiColors};
use crate::icons::Icons;
use crate::interface::Interface;
use crate::renderer::{color::Color, Renderer};

use image::{png::PngDecoder, ImageDecoder};
use winit::{
	dpi::LogicalSize,
	event::{Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	monitor::MonitorHandle,
	window::{Icon, Window, WindowBuilder},
};

pub struct App {
	window: Window,
	event_loop: EventLoop<()>,
	renderer: Renderer,
	interface: Interface,
	config: Config,
	icons: Icons,
}

impl App {
	pub fn new() -> App {
		let config = Config {
			ui_colors: UiColors {
				bg: Color::from_rgb_u32(0x282C34),
				secondary_bg: Color::from_rgb_u32(0x1D2026),
				focused_bg: Color::from_rgb_u32(0x333842),
				unfocused_bg: Color::from_rgb_u32(0x1D2026),
				text: Color::from_rgb_u32(0xC1C8D6),
				unfocused_text: Color::from_rgb_u32(0x8C919C),
				accent: Color::from_rgb_u32(0x61AFEF),
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
			},
		};

		let icons = Icons::default();

		let event_loop = EventLoop::new();

		let window = WindowBuilder::new()
			.with_title("Evalvana")
			.with_inner_size(LogicalSize::new(1400, 900))
			.with_min_inner_size(LogicalSize::new(500, 300))
			.build(&event_loop)
			.unwrap();

		App::set_scaled_icon(&window, &icons);

		let renderer = Renderer::new(&window);

		let mut interface = Interface::default();

		{
			use crate::interface::*;

			let pane1 = Pane::new("Some REPL".to_string());
			let pane2 = Pane::new("Another REPL".to_string());
			let pane3 = Pane::new("Third REPL".to_string());
			let pane4 = Pane::new("Fourth REPL".to_string());
			let pane5 = Pane::new("Fifth REPL".to_string());
			let pane6 = Pane::new("Sixth REPL".to_string());

			interface.tree_pane.pane_statuses = PaneStatuses {
				focused: 1,
				pane_statuses: vec![
					PaneStatus::of_pane(&pane1),
					PaneStatus::of_pane(&pane2),
					PaneStatus::of_pane(&pane3),
					PaneStatus::of_pane(&pane4),
					PaneStatus::of_pane(&pane5),
					PaneStatus::of_pane(&pane6),
				],
			};

			interface.tree_pane.evaluators = Evaluators {
				evaluators: vec![
					Evaluator {
						name: "Rust".to_string(),
					},
					Evaluator {
						name: "Lua".to_string(),
					},
					Evaluator {
						name: "TypeScript".to_string(),
					},
				],
			};

			interface.panes = Panes::VerticalSplit(PaneList {
				focused: 0,
				panes: vec![
					Panes::Tabbed(PaneList {
						focused: 1,
						panes: vec![Panes::Single(pane1), Panes::Single(pane2)],
					}),
					Panes::HorizontalSplit(PaneList {
						focused: 0,
						panes: vec![
							Panes::Tabbed(PaneList {
								focused: 0,
								panes: vec![
									Panes::Single(pane3),
									Panes::Single(pane4),
								],
							}),
							Panes::Tabbed(PaneList {
								focused: 1,
								panes: vec![
									Panes::Single(pane5),
									Panes::Single(pane6),
								],
							}),
						],
					}),
				],
			});
		}

		App {
			window,
			event_loop,
			renderer,
			interface,
			config,
			icons,
		}
	}

	pub fn run(self) {
		let App {
			window,
			event_loop,
			mut renderer,
			mut interface,
			config,
			mut icons,
		} = self;

		let mut monitor = window.current_monitor();
		let mut delta = App::target_delta(&monitor);

		event_loop.run(move |event, _, control_flow| {
			*control_flow = ControlFlow::Poll;

			match event {
				Event::MainEventsCleared => window.request_redraw(),

				Event::WindowEvent {
					event: WindowEvent::Resized(size),
					..
				} => {
					renderer.resize(size);
				}
				Event::WindowEvent {
					event: WindowEvent::Moved(_),
					..
				} => {
					if window.current_monitor() != monitor {
						monitor = window.current_monitor();
						delta = App::target_delta(&monitor);
					}
				}
				Event::WindowEvent {
					event: WindowEvent::ScaleFactorChanged { scale_factor, .. },
					..
				} => {
					icons.set_scale_factor(scale_factor);
					App::set_scaled_icon(&window, &icons);
				}

				Event::RedrawRequested(_) => {
					renderer.redraw(&window, &config, &mut interface);
				}

				Event::WindowEvent {
					event: WindowEvent::CloseRequested,
					window_id,
				} if window_id == window.id() => *control_flow = ControlFlow::Exit,
				_ => (),
			}
		});
	}

	fn target_delta(monitor: &MonitorHandle) -> u16 {
		monitor
			.video_modes()
			.map(|video_mode| video_mode.refresh_rate())
			.max()
			.map_or(16, |refresh_rate| {
				(1000.0 / refresh_rate as f32).floor() as u16
			})
	}

	fn set_scaled_icon(window: &Window, icons: &Icons) {
		window.set_window_icon(Some(icons.create_window_icon()));

		#[cfg(target_os = "windows")]
		{
			use winit::platform::windows::WindowExtWindows;

			window.set_taskbar_icon(Some(icons.create_taskbar_icon()));
		}
	}
}
