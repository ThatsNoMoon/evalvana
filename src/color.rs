// Copyright 2020 Benjamin Scherer
// Licensed under the Open Software License version 3.0

use iced::Color;

pub trait ColorExt {
	fn from_rgb32(rgb: u32) -> Self;
}

impl ColorExt for Color {
	fn from_rgb32(rgb: u32) -> Self {
		let (r, g, b) = (
			((rgb >> 16) & 0xFF) as u8,
			((rgb >> 8) & 0xFF) as u8,
			(rgb & 0xFF) as u8,
		);

		Self::from_rgb8(r, g, b)
	}
}
