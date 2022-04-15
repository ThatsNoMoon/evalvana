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
	use iced::{button::Style, Background};

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
}

pub(crate) mod tab {
	use iced::{
		button::{Style, StyleSheet as ButtonStyleSheet},
		Background, Color,
	};

	use crate::config::{Config, UiColors};

	pub(crate) struct Active {
		ui_colors: UiColors,
		editor_bg: Color,
	}

	pub(crate) struct Inactive {
		ui_colors: UiColors,
	}

	impl From<&'_ Config> for Active {
		fn from(config: &Config) -> Self {
			Self {
				ui_colors: config.ui_colors.clone(),
				editor_bg: config.editor_colors.bg,
			}
		}
	}

	impl From<&'_ Config> for Inactive {
		fn from(config: &Config) -> Self {
			Self {
				ui_colors: config.ui_colors.clone(),
			}
		}
	}

	impl ButtonStyleSheet for Active {
		fn active(&self) -> Style {
			Style {
				background: Some(Background::Color(self.editor_bg)),
				text_color: self.ui_colors.text,
				..Style::default()
			}
		}
	}

	impl ButtonStyleSheet for Inactive {
		fn active(&self) -> Style {
			Style {
				background: Some(Background::Color(
					self.ui_colors.unfocused_bg,
				)),
				text_color: self.ui_colors.unfocused_text,
				..Style::default()
			}
		}

		fn hovered(&self) -> Style {
			Style {
				background: Some(Background::Color(self.ui_colors.hovered_bg)),
				text_color: self.ui_colors.unfocused_text,
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
}
