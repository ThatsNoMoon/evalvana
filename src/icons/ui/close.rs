use crate::{geometry::TexPixelSize, interface::Theme};

const CLOSE_DARK_14: &'static [u8] = include_bytes!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/assets/icons/ui/close/close_dark_14.png"
));
const CLOSE_DARK_28: &'static [u8] = include_bytes!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/assets/icons/ui/close/close_dark_28.png"
));
const CLOSE_DARK_56: &'static [u8] = include_bytes!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/assets/icons/ui/close/close_dark_56.png"
));
const CLOSE_LIGHT_14: &'static [u8] = include_bytes!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/assets/icons/ui/close/close_light_14.png"
));
const CLOSE_LIGHT_28: &'static [u8] = include_bytes!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/assets/icons/ui/close/close_light_28.png"
));
const CLOSE_LIGHT_56: &'static [u8] = include_bytes!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/assets/icons/ui/close/close_light_56.png"
));

pub(in crate::icons) fn bytes_for(
	scale_factor: u8,
	theme: Theme,
) -> (&'static [u8], TexPixelSize) {
	match (theme, scale_factor) {
		(Theme::Dark, 1) => (CLOSE_DARK_14, TexPixelSize::new(14, 14)),
		(Theme::Dark, 2) => (CLOSE_DARK_28, TexPixelSize::new(28, 28)),
		(Theme::Dark, _) => (CLOSE_DARK_56, TexPixelSize::new(56, 56)),
		(Theme::Light, 1) => (CLOSE_LIGHT_14, TexPixelSize::new(14, 14)),
		(Theme::Light, 2) => (CLOSE_LIGHT_28, TexPixelSize::new(28, 28)),
		(Theme::Light, _) => (CLOSE_LIGHT_56, TexPixelSize::new(56, 56)),
	}
}
