use crate::rendering::color::Color;

use wgpu_glyph::Scale as FontScale;

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
	pub ui_colors: UiColors,
	pub editor_colors: EditorColors,
	pub font_settings: FontSettings,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiColors {
	pub bg: Color,
	pub secondary_bg: Color,
	pub focused_bg: Color,
	pub unfocused_bg: Color,
	pub text: Color,
	pub unfocused_text: Color,
	pub accent: Color,
	pub borders: Color,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorColors {
	pub bg: Color,
	pub main: Color,
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

#[derive(Debug, Clone, PartialEq)]
pub struct FontSettings {
	pub ui_font_scale: FontScale,
	pub editor_font_scale: FontScale,
}
