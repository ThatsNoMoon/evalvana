mod logo;
mod ui;

use crate::interface::Theme;

use image::{png::PngDecoder, ImageDecoder};
use winit::window::Icon;

pub struct Icons {
	scale_factor: u8,
	theme: Theme,
}

impl Default for Icons {
	fn default() -> Icons {
		Icons::new(1.0, Theme::default())
	}
}

impl Icons {
	pub fn new(scale_factor: f64, theme: Theme) -> Icons {
		Icons {
			scale_factor: norm_scale_factor(scale_factor),
			theme,
		}
	}

	pub fn create_window_icon(&self) -> Icon {
		icon_from_bytes(logo::for_scale_factor(self.scale_factor))
	}

	pub fn set_scale_factor(&mut self, scale_factor: f64) {
		self.scale_factor = norm_scale_factor(scale_factor);
	}
}

#[cfg(target_os = "windows")]
impl Icons {
	pub fn create_taskbar_icon(&self) -> Icon {
		icon_from_bytes(logo::for_scale_factor(self.scale_factor * 2))
	}
}

fn icon_from_bytes(icon_bytes: &[u8]) -> Icon {
	let logo_decoder = PngDecoder::new(icon_bytes).unwrap();
	let (logo_w, logo_h) = logo_decoder.dimensions();
	let mut logo_pixels = vec![0; logo_decoder.total_bytes() as usize];
	logo_decoder.read_image(logo_pixels.as_mut_slice()).unwrap();

	Icon::from_rgba(logo_pixels, logo_w, logo_h).unwrap()
}

fn norm_scale_factor(scale_factor: f64) -> u8 {
	(scale_factor.ceil() as u8)
		.checked_next_power_of_two()
		.unwrap_or_else(|| panic!("Invalid scale factor {}", scale_factor))
}
