// Copyright 2022 ThatsNoMoon
// Licensed under the Open Software License version 3.0

pub(crate) mod text_input {
	use evalvana_editor::style::Style;
	pub(crate) use evalvana_editor::style::StyleSheet as TextInputStyleSheet;
	use iced::{Background, Color};

	use crate::config::Config;

	pub(crate) struct Editor {
		bg: Color,
		text: Color,
		selection: Color,
		cursor: Color,
	}

	impl From<&'_ Config> for Editor {
		fn from(config: &Config) -> Self {
			Self {
				bg: config.editor_colors.bg,
				text: config.editor_colors.main,
				selection: config.editor_colors.selection,
				cursor: config.editor_colors.cursor,
			}
		}
	}

	impl TextInputStyleSheet for Editor {
		fn active(&self) -> Style {
			Style {
				background: Background::Color(self.bg),
				border_radius: 0.0,
				border_width: 0.0,
				border_color: Color::TRANSPARENT,
			}
		}

		fn focused(&self) -> Style {
			self.active()
		}

		fn placeholder_color(&self) -> iced::Color {
			self.text
		}

		fn value_color(&self) -> iced::Color {
			self.text
		}

		fn selection_color(&self) -> iced::Color {
			self.selection
		}

		fn cursor_color(&self) -> Color {
			self.cursor
		}

		fn hovered(&self) -> Style {
			self.active()
		}
	}
}

pub(crate) mod button {
	pub(crate) use iced::button::StyleSheet as ButtonStyleSheet;
	use iced::{button::Style, Background, Color};

	use crate::config::Config;

	pub(crate) struct StyleSheet {
		bg: Color,
		hovered_bg: Color,
		disabled_bg: Color,
		text: Color,
		hovered_text: Color,
		disabled_text: Color,
		border_radius: f32,
	}

	impl ButtonStyleSheet for StyleSheet {
		fn active(&self) -> Style {
			Style {
				background: Some(Background::Color(self.bg)),
				text_color: self.text,
				border_radius: self.border_radius,
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

		fn disabled(&self) -> Style {
			Style {
				background: Some(Background::Color(self.disabled_bg)),
				text_color: self.disabled_text,
				..self.active()
			}
		}

		fn pressed(&self) -> Style {
			Style {
				background: Some(Background::Color(self.hovered_bg)),
				text_color: self.hovered_text,
				..self.active()
			}
		}
	}

	pub(crate) fn primary(config: &Config) -> StyleSheet {
		StyleSheet {
			bg: config.ui_colors.secondary_bg,
			hovered_bg: config.ui_colors.hovered_bg,
			disabled_bg: config.ui_colors.unfocused_bg,
			text: config.ui_colors.text,
			hovered_text: config.ui_colors.text,
			disabled_text: config.ui_colors.unfocused_text,
			border_radius: 1.0,
		}
	}

	pub(crate) fn new_cell(config: &Config) -> StyleSheet {
		StyleSheet {
			bg: config.editor_colors.bg,
			text: config.ui_colors.unfocused_icon,
			hovered_text: config.ui_colors.text,
			..primary(config)
		}
	}

	pub(crate) fn tab_close(config: &Config, is_active: bool) -> StyleSheet {
		StyleSheet {
			bg: if is_active {
				config.editor_colors.bg
			} else {
				config.ui_colors.unfocused_bg
			},
			text: config.ui_colors.unfocused_text,
			hovered_text: config.ui_colors.text,
			..primary(config)
		}
	}

	pub(crate) fn tab_handle(config: &Config) -> StyleSheet {
		StyleSheet {
			bg: config.ui_colors.unfocused_bg,
			disabled_bg: config.editor_colors.bg,
			text: config.ui_colors.unfocused_text,
			disabled_text: config.ui_colors.text,
			hovered_text: config.ui_colors.text,
			..primary(config)
		}
	}
}

pub(crate) mod container {
	use iced::{
		container::{Style, StyleSheet as ContainerStyleSheet},
		Background, Color,
	};

	use crate::config::Config;

	pub(crate) struct StyleSheet {
		bg: Color,
	}

	impl ContainerStyleSheet for StyleSheet {
		fn style(&self) -> Style {
			Style {
				background: Some(Background::Color(self.bg)),
				..Style::default()
			}
		}
	}

	pub(crate) fn ui_bg(config: &Config) -> StyleSheet {
		StyleSheet {
			bg: config.ui_colors.bg,
		}
	}

	pub(crate) fn editor_bg(config: &Config) -> StyleSheet {
		StyleSheet {
			bg: config.editor_colors.bg,
		}
	}

	pub(crate) fn secondary_bg(config: &Config) -> StyleSheet {
		StyleSheet {
			bg: config.ui_colors.secondary_bg,
		}
	}
}

pub(crate) mod rule {
	use iced::{
		rule::{FillMode, Style, StyleSheet as RuleStyleSheet},
		Color,
	};

	use crate::config::Config;

	pub(crate) struct StyleSheet {
		color: Color,
		width: u16,
		fill_mode: FillMode,
	}

	impl RuleStyleSheet for StyleSheet {
		fn style(&self) -> Style {
			Style {
				color: self.color,
				width: self.width,
				fill_mode: self.fill_mode,
				..Style::default()
			}
		}
	}

	pub(crate) fn cell_divider(config: &Config, width: u16) -> StyleSheet {
		StyleSheet {
			color: config.ui_colors.borders,
			width,
			fill_mode: FillMode::Full,
		}
	}

	pub(crate) fn tab_divider(config: &Config, width: u16) -> StyleSheet {
		StyleSheet {
			color: config.ui_colors.borders,
			width,
			fill_mode: FillMode::Percent(75.0),
		}
	}
}
