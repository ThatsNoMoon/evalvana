use crate::renderer::color::Color;

pub struct Config {
    pub ui_colors: UiColors,
    pub editor_colors: EditorColors,
}

pub struct UiColors {
    pub bg: Color,
    pub text: Color,
    pub borders: Color,
}

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
