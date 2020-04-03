use wgpu_glyph::Section;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundingBox {
	pub x: u32,
	pub y: u32,
	pub w: u32,
	pub h: u32,
}

impl BoundingBox {
	pub fn new(x: u32, y: u32, w: u32, h: u32) -> BoundingBox {
		BoundingBox { x, y, w, h }
	}

	pub fn to_section_bounds(self) -> Section<'static> {
		let BoundingBox { x, y, w, h } = self;
		Section {
			screen_position: (x as f32, y as f32),
			bounds: (w as f32, h as f32),
			..Section::default()
		}
	}

	pub fn with_x(mut self, x: u32) -> BoundingBox {
		self.x = x;
		self
	}

	pub fn with_y(mut self, y: u32) -> BoundingBox {
		self.y = y;
		self
	}

	pub fn with_w(mut self, w: u32) -> BoundingBox {
		self.w = w;
		self
	}

	pub fn with_h(mut self, h: u32) -> BoundingBox {
		self.h = h;
		self
	}

	pub fn with_left(mut self, left: u32) -> BoundingBox {
		if left > self.x {
			self.w -= left - self.x;
		} else {
			self.w += self.x - left;
		}
		self.x = left;
		self
	}

	pub fn with_right(mut self, right: u32) -> BoundingBox {
		self.w = right - self.x;
		self
	}

	pub fn with_top(mut self, top: u32) -> BoundingBox {
		if top > self.y {
			self.h -= top - self.y;
		} else {
			self.h += self.y - top;
		}
		self.y = top;
		self
	}

	pub fn with_bottom(mut self, bot: u32) -> BoundingBox {
		self.h = bot - self.y;
		self
	}

	pub fn added_x(mut self, x: u32) -> BoundingBox {
		self.x += x;
		self
	}

	pub fn added_y(mut self, y: u32) -> BoundingBox {
		self.y += y;
		self
	}

	pub fn added_w(mut self, w: u32) -> BoundingBox {
		self.w += w;
		self
	}

	pub fn added_h(mut self, h: u32) -> BoundingBox {
		self.h += h;
		self
	}

	pub fn subbed_x(mut self, x: u32) -> BoundingBox {
		self.x -= x;
		self
	}

	pub fn subbed_y(mut self, y: u32) -> BoundingBox {
		self.y -= y;
		self
	}

	pub fn subbed_w(mut self, w: u32) -> BoundingBox {
		self.w -= w;
		self
	}

	pub fn subbed_h(mut self, h: u32) -> BoundingBox {
		self.h -= h;
		self
	}

	pub fn added_left(mut self, amnt: u32) -> BoundingBox {
		self.x += amnt;
		self.w -= amnt;
		self
	}

	pub fn added_right(mut self, amnt: u32) -> BoundingBox {
		self.w += amnt;
		self
	}

	pub fn added_top(mut self, amnt: u32) -> BoundingBox {
		self.y += amnt;
		self.h -= amnt;
		self
	}

	pub fn added_bottom(mut self, amnt: u32) -> BoundingBox {
		self.h += amnt;
		self
	}

	pub fn subbed_left(mut self, amnt: u32) -> BoundingBox {
		self.x -= amnt;
		self.w += amnt;
		self
	}

	pub fn subbed_right(mut self, amnt: u32) -> BoundingBox {
		self.w -= amnt;
		self
	}

	pub fn subbed_top(mut self, amnt: u32) -> BoundingBox {
		self.y -= amnt;
		self.h += amnt;
		self
	}

	pub fn subbed_bottom(mut self, amnt: u32) -> BoundingBox {
		self.h -= amnt;
		self
	}

	pub fn top(&self) -> u32 {
		self.y
	}

	pub fn bottom(&self) -> u32 {
		self.y + self.h
	}

	pub fn left(&self) -> u32 {
		self.x
	}

	pub fn right(&self) -> u32 {
		self.x + self.w
	}
}
