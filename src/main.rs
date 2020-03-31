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



fn main() {
    env_logger::init();

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
    let size = window.inner_size();

    let mut renderer = Renderer::new(&window, size);

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

