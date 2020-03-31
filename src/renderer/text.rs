use std::borrow::Cow;

use wgpu_glyph::{
    Section,
    GlyphBrush,
    GlyphBrushBuilder,
    Scale as FontScale,
    SectionText,
    VariedSection,
};

pub struct TextRenderer {
	glyph_brush: GlyphBrush<'static, ()>,
}

impl TextRenderer {
	pub fn new(device: &mut wgpu::Device, font: &'static [u8], texture_format: wgpu::TextureFormat) -> TextRenderer {
		TextRenderer {
			glyph_brush: GlyphBrushBuilder::using_font_bytes(font).build(device, texture_format),
		}
	}

	pub fn queue<'a>(&mut self, section: impl Into<Cow<'a, VariedSection<'a>>>) {
		self.glyph_brush.queue(section);
	}

	pub fn draw_queued(&mut self, device: &mut wgpu::Device, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView, target_width: u32, target_height: u32) -> Result<(), String> {
		self.glyph_brush.draw_queued(device, encoder, target, target_width, target_height)
	}
}