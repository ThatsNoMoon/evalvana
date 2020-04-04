pub mod dark {
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

	pub fn for_scale_factor(scale_factor: u8) -> &'static [u8] {
		match scale_factor {
			1 => CLOSE_DARK_14,
			2 => CLOSE_DARK_28,
			_ => CLOSE_DARK_56,
		}
	}
}

pub mod light {
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

	pub fn for_scale_factor(scale_factor: u8) -> &'static [u8] {
		match scale_factor {
			1 => CLOSE_LIGHT_14,
			2 => CLOSE_LIGHT_28,
			_ => CLOSE_LIGHT_56,
		}
	}
}
