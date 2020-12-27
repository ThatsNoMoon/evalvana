pub mod button {
	use crate::config::{Config, UiColors};
	use iced::{button::Style, Background};

	pub use iced::button::StyleSheet as ButtonStyleSheet;

	pub struct Primary {
		ui_colors: UiColors,
	}

	pub struct Secondary {
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

pub mod tab {
	use crate::config::{Config, UiColors};
	use iced::{
		button::{Style, StyleSheet as ButtonStyleSheet},
		Background, Color,
	};

	pub struct Active {
		ui_colors: UiColors,
		editor_bg: Color,
	}

	pub struct Inactive {
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

pub mod container {
	use crate::config::Config;
	use iced::{
		container::{Style, StyleSheet as ContainerStyleSheet},
		Background, Color,
	};

	pub struct TabBg {
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

	pub struct UiBg {
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
}
