// Copyright 2020 Benjamin Scherer
// Licensed under the Open Software License version 3.0

use std::borrow::Cow;

use wgpu_glyph::{
	ab_glyph::FontRef, FontId, GlyphBrush, GlyphBrushBuilder, Section,
};

const EDITOR_FONT_BYTES: &[u8] = include_bytes!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/assets/fonts/JetBrainsMono/JetBrainsMono-Regular.ttf"
));

const UI_FONT_BYTES: &[u8] = include_bytes!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/assets/fonts/Roboto/Roboto-Regular.ttf"
));

const UI_FONT_MEDIUM_BYTES: &[u8] = include_bytes!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/assets/fonts/Roboto/Roboto-Medium.ttf"
));

const UI_FONT_BOLD_BYTES: &[u8] = include_bytes!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/assets/fonts/Roboto/Roboto-Bold.ttf"
));

const UI_FONT_ITALIC_BYTES: &[u8] = include_bytes!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/assets/fonts/Roboto/Roboto-Italic.ttf"
));

pub struct TextRenderer {
	glyph_brush: GlyphBrush<(), FontRef<'static>>,
	editor_font: FontId,
	ui_font: FontId,
	ui_font_medium: FontId,
	ui_font_bold: FontId,
	ui_font_italic: FontId,
}

impl TextRenderer {
	pub fn new(
		device: &mut wgpu::Device,
		texture_format: wgpu::TextureFormat,
	) -> TextRenderer {
		let editor_font = FontRef::try_from_slice(EDITOR_FONT_BYTES).unwrap();
		let ui_font = FontRef::try_from_slice(UI_FONT_BYTES).unwrap();
		let ui_font_medium =
			FontRef::try_from_slice(UI_FONT_MEDIUM_BYTES).unwrap();
		let ui_font_bold = FontRef::try_from_slice(UI_FONT_BOLD_BYTES).unwrap();
		let ui_font_italic =
			FontRef::try_from_slice(UI_FONT_ITALIC_BYTES).unwrap();
		let mut builder = GlyphBrushBuilder::using_font(editor_font);

		let ui_font = builder.add_font(ui_font);
		let ui_font_medium = builder.add_font(ui_font_medium);
		let ui_font_bold = builder.add_font(ui_font_bold);
		let ui_font_italic = builder.add_font(ui_font_italic);

		TextRenderer {
			glyph_brush: builder.build(device, texture_format),
			editor_font: FontId(0),
			ui_font,
			ui_font_bold,
			ui_font_medium,
			ui_font_italic,
		}
	}

	pub fn queue<'a>(&mut self, section: impl Into<Cow<'a, Section<'a>>>) {
		self.glyph_brush.queue(section);
	}

	pub fn draw_queued(
		&mut self,
		device: &mut wgpu::Device,
		encoder: &mut wgpu::CommandEncoder,
		target: &wgpu::TextureView,
		target_width: u32,
		target_height: u32,
	) -> Result<(), String> {
		self.glyph_brush.draw_queued(
			device,
			encoder,
			target,
			target_width,
			target_height,
		)
	}

	pub fn editor_font(&self) -> FontId {
		self.editor_font
	}

	pub fn ui_font(&self) -> FontId {
		self.ui_font
	}

	pub fn ui_font_medium(&self) -> FontId {
		self.ui_font_medium
	}

	pub fn ui_font_bold(&self) -> FontId {
		self.ui_font_bold
	}

	pub fn ui_font_italic(&self) -> FontId {
		self.ui_font_italic
	}

	pub fn font_data(&self, id: FontId) -> &FontRef {
		&self.glyph_brush.fonts()[id.0]
	}
}
