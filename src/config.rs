use crate::renderer::color::Color;

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
	pub ui_colors: UiColors,
	pub editor_colors: EditorColors,
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
	pub strings: Color,
	pub numbers: Color,
	pub operators: Color,
	pub keywords: Color,
	pub variables: Color,
	pub parameters: Color,
	pub constants: Color,
	pub types: Color,
	pub functions: Color,
}
