// Copyright 2022 ThatsNoMoon
// Licensed under the Open Software License version 3.0

pub(crate) mod text_input {
	use evalvana_editor::style::Style;
	pub(crate) use evalvana_editor::style::StyleSheet as TextInputStyleSheet;
	use iced::{Background, Color};

	use crate::config::{Config, EditorColors};

	pub(crate) struct Editor {
		colors: EditorColors,
	}

	impl From<&'_ Config> for Editor {
		fn from(config: &Config) -> Self {
			Self {
				colors: config.editor_colors.clone(),
			}
		}
	}

	impl TextInputStyleSheet for Editor {
		fn active(&self) -> Style {
			Style {
				background: Background::Color(self.colors.bg),
				border_radius: 0.0,
				border_width: 0.0,
				border_color: Color::TRANSPARENT,
			}
		}

		fn focused(&self) -> Style {
			self.active()
		}

		fn placeholder_color(&self) -> iced::Color {
			self.colors.main
		}

		fn value_color(&self) -> iced::Color {
			self.colors.main
		}

		fn selection_color(&self) -> iced::Color {
			self.colors.selection
		}

		fn cursor_color(&self) -> Color {
			self.colors.cursor
		}

		fn hovered(&self) -> Style {
			self.active()
		}
	}
}

pub(crate) mod button {
	pub(crate) use iced::button::StyleSheet as ButtonStyleSheet;
	use iced::{button::Style, Background, Color};

	use crate::config::{Config, UiColors};

	pub(crate) struct Primary {
		ui_colors: UiColors,
	}

	pub(crate) struct Secondary {
		ui_colors: UiColors,
	}

	impl From<&'_ Config> for Primary {
		fn from(config: &Config) -> Self {
			Self {
				ui_colors: config.ui_colors.clone(),
			}
		}
	}

	impl From<&'_ Config> for Secondary {
		fn from(config: &Config) -> Self {
			Self {
				ui_colors: config.ui_colors.clone(),
			}
		}
	}

	impl ButtonStyleSheet for Primary {
		fn active(&self) -> Style {
			Style {
				background: Some(Background::Color(
					self.ui_colors.secondary_bg,
				)),
				text_color: self.ui_colors.text,
				border_radius: 1.0,
				..Style::default()
			}
		}

		fn hovered(&self) -> Style {
			Style {
				background: Some(Background::Color(self.ui_colors.hovered_bg)),
				..self.active()
			}
		}

		fn disabled(&self) -> Style {
			Style {
				background: Some(Background::Color(
					self.ui_colors.unfocused_bg,
				)),
				text_color: self.ui_colors.unfocused_text,
				..self.active()
			}
		}
	}

	impl ButtonStyleSheet for Secondary {
		fn active(&self) -> Style {
			Style {
				background: Some(Background::Color(self.ui_colors.bg)),
				text_color: self.ui_colors.text,
				..Style::default()
			}
		}

		fn hovered(&self) -> Style {
			Style {
				background: Some(Background::Color(self.ui_colors.hovered_bg)),
				..self.active()
			}
		}

		fn disabled(&self) -> Style {
			Style {
				background: Some(Background::Color(
					self.ui_colors.secondary_unfocused_bg,
				)),
				text_color: self.ui_colors.unfocused_text,
				..self.active()
			}
		}
	}

	pub(crate) struct NewCell {
		bg: Color,
		hovered_bg: Color,
		text_color: Color,
		hovered_text_color: Color,
	}

	impl From<&'_ Config> for NewCell {
		fn from(config: &Config) -> Self {
			Self {
				bg: config.ui_colors.bg,
				hovered_bg: config.ui_colors.hovered_bg,
				text_color: config.ui_colors.unfocused_icon,
				hovered_text_color: config.ui_colors.text,
			}
		}
	}

	impl ButtonStyleSheet for NewCell {
		fn active(&self) -> Style {
			Style {
				background: Some(Background::Color(self.bg)),
				text_color: self.text_color,
				..Style::default()
			}
		}

		fn hovered(&self) -> Style {
			Style {
				background: Some(Background::Color(self.hovered_bg)),
				text_color: self.hovered_text_color,
				..self.active()
			}
		}
	}

	pub(crate) struct TabClose {
		bg: Color,
		hovered_bg: Color,
		text: Color,
		unfocused_text: Color,
	}

	impl TabClose {
		pub(crate) fn new(config: &Config, is_active: bool) -> Self {
			Self {
				bg: if is_active {
					config.editor_colors.bg
				} else {
					config.ui_colors.unfocused_bg
				},
				hovered_bg: config.ui_colors.hovered_bg,
				text: config.ui_colors.text,
				unfocused_text: config.ui_colors.unfocused_text,
			}
		}
	}

	impl ButtonStyleSheet for TabClose {
		fn active(&self) -> Style {
			Style {
				background: Some(Background::Color(self.bg)),
				text_color: self.unfocused_text,
				..Style::default()
			}
		}

		fn hovered(&self) -> Style {
			Style {
				background: Some(Background::Color(self.hovered_bg)),
				text_color: self.text,
				..self.active()
			}
		}
	}

	pub(crate) struct TabHandle {
		bg: Color,
		hovered_bg: Color,
		text: Color,
		hovered_text: Color,
	}

	impl TabHandle {
		pub(crate) fn new(config: &Config, is_active: bool) -> Self {
			Self {
				bg: if is_active {
					config.editor_colors.bg
				} else {
					config.ui_colors.unfocused_bg
				},
				hovered_bg: config.ui_colors.hovered_bg,
				text: if is_active {
					config.ui_colors.text
				} else {
					config.ui_colors.unfocused_text
				},
				hovered_text: config.ui_colors.text,
			}
		}
	}

	impl ButtonStyleSheet for TabHandle {
		fn active(&self) -> Style {
			Style {
				background: Some(Background::Color(self.bg)),
				text_color: self.text,
				..Style::default()
			}
		}

		fn hovered(&self) -> Style {
			Style {
				background: Some(Background::Color(self.hovered_bg)),
				text_color: self.hovered_text,
				..self.active()
			}
		}
	}
}

pub(crate) mod container {
	use iced::{
		container::{Style, StyleSheet as ContainerStyleSheet},
		Background, Color,
	};

	use crate::config::Config;

	pub(crate) struct TabBg {
		bg: Color,
	}

	impl From<&'_ Config> for TabBg {
		fn from(config: &Config) -> Self {
			Self {
				bg: config.ui_colors.borders,
			}
		}
	}

	impl ContainerStyleSheet for TabBg {
		fn style(&self) -> Style {
			Style {
				background: Some(Background::Color(self.bg)),
				..Style::default()
			}
		}
	}

	pub(crate) struct UiBg {
		bg: Color,
	}

	impl From<&'_ Config> for UiBg {
		fn from(config: &Config) -> Self {
			Self {
				bg: config.ui_colors.bg,
			}
		}
	}

	impl ContainerStyleSheet for UiBg {
		fn style(&self) -> Style {
			Style {
				background: Some(Background::Color(self.bg)),
				..Style::default()
			}
		}
	}

	pub(crate) struct SecondaryBg {
		bg: Color,
	}

	impl From<&'_ Config> for SecondaryBg {
		fn from(config: &Config) -> Self {
			Self {
				bg: config.ui_colors.secondary_bg,
			}
		}
	}

	impl ContainerStyleSheet for SecondaryBg {
		fn style(&self) -> Style {
			Style {
				background: Some(Background::Color(self.bg)),
				..Style::default()
			}
		}
	}

	pub(crate) struct Cell {
		bg: Color,
		border_color: Color,
	}

	impl From<&'_ Config> for Cell {
		fn from(config: &Config) -> Self {
			Self {
				bg: config.ui_colors.bg,
				border_color: config.ui_colors.borders,
			}
		}
	}

	impl ContainerStyleSheet for Cell {
		fn style(&self) -> Style {
			Style {
				background: Some(Background::Color(self.bg)),
				border_color: self.border_color,
				border_width: 2.0,
				border_radius: 8.0,
				..Style::default()
			}
		}
	}
}

pub(crate) mod rule {
	use iced::{
		rule::{Style, StyleSheet as RuleStyleSheet},
		Color,
	};

	use crate::config::Config;

	pub(crate) struct CellDivider {
		color: Color,
		width: u16,
	}

	impl CellDivider {
		pub(crate) fn new(config: &Config, width: u16) -> Self {
			Self {
				color: config.ui_colors.borders,
				width,
			}
		}
	}

	impl From<&'_ Config> for CellDivider {
		fn from(config: &Config) -> Self {
			Self {
				color: config.ui_colors.borders,
				width: 1,
			}
		}
	}

	impl RuleStyleSheet for CellDivider {
		fn style(&self) -> Style {
			Style {
				color: self.color,
				width: self.width,
				..Style::default()
			}
		}
	}
}
