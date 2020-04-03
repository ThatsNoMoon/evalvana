use std::borrow::Cow;

use wgpu_glyph::{
	FontId, GlyphBrush, GlyphBrushBuilder, Scale as FontScale, Section,
	SectionText, VariedSection,
};

const EDITOR_FONT: &'static [u8] = include_bytes!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/assets/fonts/JetBrainsMono/JetBrainsMono-Regular.ttf"
));

const UI_FONT: &'static [u8] = include_bytes!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/assets/fonts/Roboto/Roboto-Regular.ttf"
));

pub struct TextRenderer {
	glyph_brush: GlyphBrush<'static, ()>,
	editor_font: FontId,
	ui_font: FontId,
}

impl TextRenderer {
	pub fn new(
		device: &mut wgpu::Device,
		texture_format: wgpu::TextureFormat,
	) -> TextRenderer {
		let mut builder =
			GlyphBrushBuilder::using_font_bytes(EDITOR_FONT).unwrap();
		let ui_font = builder.add_font_bytes(UI_FONT);
		TextRenderer {
			glyph_brush: builder.build(device, texture_format),
			editor_font: FontId(0),
			ui_font,
		}
	}

	pub fn queue<'a>(
		&mut self,
		section: impl Into<Cow<'a, VariedSection<'a>>>,
	) {
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
}
