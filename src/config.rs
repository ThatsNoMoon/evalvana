// Copyright 2021 ThatsNoMoon
// Licensed under the Open Software License version 3.0

use iced::Color;

use crate::color::ColorExt;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Config {
	pub ui_colors: UiColors,
	pub editor_colors: EditorColors,
	pub text_settings: TextSettings,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiColors {
	pub bg: Color,
	pub secondary_bg: Color,
	pub hovered_bg: Color,
	pub focused_bg: Color,
	pub unfocused_bg: Color,
	pub secondary_unfocused_bg: Color,
	pub text: Color,
	pub unfocused_text: Color,
	pub accent: Color,
	pub borders: Color,
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
pub struct EditorColors {
	pub bg: Color,
	pub main: Color,
	pub selection: Color,
	pub gutter: Color,
	pub strings: Color,
	pub numbers: Color,
	pub operators: Color,
	pub keywords: Color,
	pub variables: Color,
	pub parameters: Color,
	pub constants: Color,
	pub types: Color,
	pub functions: Color,

	pub success: Color,
	pub warnings: Color,
	pub errors: Color,
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
pub struct TextSettings {
	pub ui_font_size: u16,
	pub editor_font_size: u16,
	pub header_font_size: u16,
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
