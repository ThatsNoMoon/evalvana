use crate::{
	config::{Config, EditorColors, FontSettings, UiColors},
	events::Event,
	icons::Icons,
	interface::{Interface, UpdatingContext},
	rendering::{color::Color, Renderer},
};

use pollster::block_on;
use wgpu_glyph::ab_glyph::PxScale as FontScale;
use winit::{
	dpi::LogicalSize,
	event::{Event as WinitEvent, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	monitor::MonitorHandle,
	window::{Window, WindowBuilder},
};

pub struct App {
	window: Window,
	event_loop: EventLoop<()>,
	renderer: Renderer,
	interface: Interface,
	config: Config,
	icons: Icons,
}

impl Default for App {
	fn default() -> App {
		let config = Config {
			ui_colors: UiColors {
				bg: Color::from_rgb_u32(0x282C34),
				secondary_bg: Color::from_rgb_u32(0x1D2026),
				hovered_bg: Color::from_rgb_u32(0x2F343D),
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
				gutter: Color::from_rgb_u32(0x838891),
				strings: Color::from_rgb_u32(0x98C379),
				numbers: Color::from_rgb_u32(0xD19A66),
				operators: Color::from_rgb_u32(0xC678DD),
				keywords: Color::from_rgb_u32(0xE06C75),
				variables: Color::from_rgb_u32(0xE5C07B),
				parameters: Color::from_rgb_u32(0xE5C07B),
				constants: Color::from_rgb_u32(0x56B6C2),
				types: Color::from_rgb_u32(0x61AFEF),
				functions: Color::from_rgb_u32(0xABB2BF),
				success: Color::from_rgb_u32(0x5DD47F),
				warnings: Color::from_rgb_u32(0xEBCD2E),
				errors: Color::from_rgb_u32(0xFF4545),
			},
			font_settings: FontSettings {
				ui_font_scale: FontScale::from(16.0),
				editor_font_scale: FontScale::from(16.0),
			},
		};

		let icons = Icons::default();

		log::trace!("Creating event loop & window");

		let event_loop = EventLoop::new();

		let window = WindowBuilder::new()
			.with_title("Evalvana")
			.with_inner_size(LogicalSize::new(1400, 900))
			.with_min_inner_size(LogicalSize::new(500, 300))
			.build(&event_loop)
			.unwrap();

		App::set_scaled_icon(&window, &icons);

		log::trace!("Creating renderer");

		let mut renderer = block_on(Renderer::new(&window, &icons));

		let mut updating_ctx = UpdatingContext::new(
			&mut renderer.drawing_manager,
			&window,
			Event::Startup,
		);

		log::trace!("Creating interface");

		let mut interface = Interface::new(&mut updating_ctx);

		{
			use crate::interface::*;
			use crate::repl::evaluation::*;

			let ctx = &mut updating_ctx;

			let pane1 = Pane::new(ctx, "Some REPL".to_string());
			let mut pane2 = Pane::new(ctx, "Another REPL".to_string());
			let pane3 = Pane::new(ctx, "Third REPL".to_string());
			let pane4 = Pane::new(ctx, "Fourth REPL".to_string());
			let pane5 = Pane::new(ctx, "Fifth REPL".to_string());
			let pane6 = Pane::new(ctx, "Sixth REPL".to_string());

			pane2.evaluations = vec![Evaluation {
				input: Expression {
					input: "let x = 2;\nlet y = 2;\nx * y".to_string(),
				},
				output: Result::Success(PlainResult {
					text: "4".to_string(),
				}),
			}];

			interface.tree_pane.pane_statuses = PaneStatuses {
				drawn_bounds: None,
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
				drawn_bounds: None,
				evaluators: vec![
					Evaluator {
						drawn_bounds: None,
						name: "Rust".to_string(),
						hovered: false,
					},
					Evaluator {
						drawn_bounds: None,
						name: "Lua".to_string(),
						hovered: false,
					},
					Evaluator {
						drawn_bounds: None,
						name: "TypeScript".to_string(),
						hovered: false,
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
}

impl App {
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
			log::trace!("Processing event: {:?}", event);
			*control_flow = ControlFlow::Poll;

			match &event {
				WinitEvent::MainEventsCleared => window.request_redraw(),

				WinitEvent::WindowEvent {
					event: WindowEvent::Resized(size),
					..
				} => {
					renderer.resize(*size);
				}
				WinitEvent::WindowEvent {
					event: WindowEvent::Moved(_),
					..
				} => {
					if window.current_monitor() != monitor {
						monitor = window.current_monitor();
						delta = App::target_delta(&monitor);
					}
				}
				WinitEvent::WindowEvent {
					event: WindowEvent::ScaleFactorChanged { scale_factor, .. },
					..
				} => {
					icons.set_scale_factor(*scale_factor);
					App::set_scaled_icon(&window, &icons);
				}

				WinitEvent::RedrawRequested(_) => {
					renderer.redraw(&window, &config, &icons, &mut interface);
				}

				WinitEvent::WindowEvent {
					event: WindowEvent::CloseRequested,
					window_id,
				} if window_id == &window.id() => *control_flow = ControlFlow::Exit,
				_ => (),
			}

			let mut updating_ctx = UpdatingContext::new(
				&mut renderer.drawing_manager,
				&window,
				Event::WinitEvent(event),
			);
			interface.update(&mut updating_ctx);
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
