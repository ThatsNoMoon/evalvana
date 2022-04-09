// Copyright 2021 ThatsNoMoon
// Licensed under the Open Software License version 3.0

use iced::Color;

use crate::color::ColorExt;

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct Config {
	pub(crate) ui_colors: UiColors,
	pub(crate) editor_colors: EditorColors,
	pub(crate) text_settings: TextSettings,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct UiColors {
	pub(crate) bg: Color,
	pub(crate) secondary_bg: Color,
	pub(crate) hovered_bg: Color,
	pub(crate) focused_bg: Color,
	pub(crate) unfocused_bg: Color,
	pub(crate) secondary_unfocused_bg: Color,
	pub(crate) text: Color,
	pub(crate) unfocused_text: Color,
	pub(crate) accent: Color,
	pub(crate) borders: Color,
}

impl Default for UiColors {
	fn default() -> Self {
		Self {
			bg: Color::from_rgb32(0x282C34),
			secondary_bg: Color::from_rgb32(0x1D2026),
			hovered_bg: Color::from_rgb32(0x2F343D),
			focused_bg: Color::from_rgb32(0x333842),
			unfocused_bg: Color::from_rgb32(0x1D2026),
			secondary_unfocused_bg: Color::from_rgb32(0x313640),
			text: Color::from_rgb32(0xC1C8D6),
			unfocused_text: Color::from_rgb32(0x8C919C),
			accent: Color::from_rgb32(0x61AFEF),
			borders: Color::from_rgb32(0x4B5263),
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct EditorColors {
	pub(crate) bg: Color,
	pub(crate) main: Color,
	pub(crate) selection: Color,
	pub(crate) gutter: Color,
	pub(crate) strings: Color,
	pub(crate) numbers: Color,
	pub(crate) operators: Color,
	pub(crate) keywords: Color,
	pub(crate) variables: Color,
	pub(crate) parameters: Color,
	pub(crate) constants: Color,
	pub(crate) types: Color,
	pub(crate) functions: Color,

	pub(crate) success: Color,
	pub(crate) warnings: Color,
	pub(crate) errors: Color,
}

impl Default for EditorColors {
	fn default() -> Self {
		Self {
			bg: Color::from_rgb32(0x282C34),
			main: Color::from_rgb32(0xABB2BF),
			selection: Color::from_rgba8(0x61, 0xAF, 0xEF, 0.3),
			gutter: Color::from_rgb32(0x838891),
			strings: Color::from_rgb32(0x98C379),
			numbers: Color::from_rgb32(0xD19A66),
			operators: Color::from_rgb32(0xC678DD),
			keywords: Color::from_rgb32(0xE06C75),
			variables: Color::from_rgb32(0xE5C07B),
			parameters: Color::from_rgb32(0xE5C07B),
			constants: Color::from_rgb32(0x56B6C2),
			types: Color::from_rgb32(0x61AFEF),
			functions: Color::from_rgb32(0xABB2BF),
			success: Color::from_rgb32(0x5DD47F),
			warnings: Color::from_rgb32(0xEBCD2E),
			errors: Color::from_rgb32(0xFF4545),
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TextSettings {
	pub(crate) ui_font_size: u16,
	pub(crate) editor_font_size: u16,
	pub(crate) header_font_size: u16,
}

impl Default for TextSettings {
	fn default() -> Self {
		Self {
			ui_font_size: 16,
			editor_font_size: 16,
			header_font_size: 20,
		}
	}
}
