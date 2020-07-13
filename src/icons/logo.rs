// Copyright 2020 Benjamin Scherer
// Licensed under the Open Software License version 3.0

const LOGO_16: &[u8] = include_bytes!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/assets/icons/logo/logo_16.png"
));
const LOGO_32: &[u8] = include_bytes!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/assets/icons/logo/logo_32.png"
));
const LOGO_64: &[u8] = include_bytes!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/assets/icons/logo/logo_64.png"
));
const LOGO_128: &[u8] = include_bytes!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/assets/icons/logo/logo_128.png"
));

pub fn for_scale_factor(scale_factor: u8) -> &'static [u8] {
	match scale_factor {
		1 => LOGO_16,
		2 => LOGO_32,
		4 => LOGO_64,
		_ => LOGO_128,
	}
}
