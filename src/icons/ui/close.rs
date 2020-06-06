use crate::geometry::TexPixelSize;

use winit::window::Theme;

const CLOSE_DARK_28: &[u8] = include_bytes!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/assets/icons/ui/close/close_dark_28.png"
));
const CLOSE_DARK_56: &[u8] = include_bytes!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/assets/icons/ui/close/close_dark_56.png"
));
const CLOSE_LIGHT_28: &[u8] = include_bytes!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/assets/icons/ui/close/close_light_28.png"
));
const CLOSE_LIGHT_56: &[u8] = include_bytes!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/assets/icons/ui/close/close_light_56.png"
));

pub(in crate::icons) fn bytes_for(
	scale_factor: u8,
	theme: &Theme,
) -> (&'static [u8], TexPixelSize) {
	match (theme, scale_factor) {
		(Theme::Dark, 1) => (CLOSE_DARK_28, TexPixelSize::new(28, 28)),
		(Theme::Dark, _) => (CLOSE_DARK_56, TexPixelSize::new(56, 56)),
		(Theme::Light, 1) => (CLOSE_LIGHT_28, TexPixelSize::new(28, 28)),
		(Theme::Light, _) => (CLOSE_LIGHT_56, TexPixelSize::new(56, 56)),
	}
}
