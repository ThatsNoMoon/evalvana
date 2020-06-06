use euclid::{Length, Point2D, Rect, Scale, Size2D};

pub enum ScreenPixelSpace {}
pub enum ScreenNormSpace {}
pub enum ScreenPhysicalSpace {}

pub enum TexPixelSpace {}
pub enum TexNormSpace {}
pub enum TexByteSpace {}

pub type ScreenPixelLength = Length<u32, ScreenPixelSpace>;
pub type ScreenNormLength = Length<f32, ScreenNormSpace>;
pub type ScreenPhysicalLength = Length<u32, ScreenPhysicalSpace>;

pub type ScreenPixelPoint = Point2D<u32, ScreenPixelSpace>;
pub type ScreenNormPoint = Point2D<f32, ScreenNormSpace>;
pub type ScreenPhysicalPoint = Point2D<u32, ScreenPhysicalSpace>;

pub type ScreenPixelSize = Size2D<u32, ScreenPixelSpace>;
pub type ScreenNormSize = Size2D<f32, ScreenNormSpace>;
pub type ScreenPhysicalSize = Size2D<u32, ScreenPhysicalSpace>;

pub type ScreenPixelRect = Rect<u32, ScreenPixelSpace>;
pub type ScreenNormRect = Rect<f32, ScreenNormSpace>;
pub type ScreenPhysicalRect = Rect<u32, ScreenPhysicalSpace>;

pub type TexPixelPoint = Point2D<u32, TexPixelSpace>;
pub type TexNormPoint = Point2D<f32, TexNormSpace>;
pub type TexBytePoint = Point2D<u32, TexByteSpace>;

pub type TexPixelSize = Size2D<u32, TexPixelSpace>;
pub type TexNormSize = Size2D<f32, TexNormSpace>;
pub type TexByteSize = Size2D<u32, TexByteSpace>;

pub type TexPixelRect = Rect<u32, TexPixelSpace>;
pub type TexNormRect = Rect<f32, TexNormSpace>;
pub type TexByteRect = Rect<u32, TexByteSpace>;

pub mod bounding_box_ext {
	use std::ops::{Add, AddAssign, Sub, SubAssign};

	use euclid::{Point2D, Rect, Size2D};
	use num_traits::cast::NumCast;
	use wgpu_glyph::Section;

	pub trait BoundingBoxExt<T, S>: Sized {
		fn top(&self) -> T;
		fn bottom(&self) -> T;
		fn left(&self) -> T;
		fn right(&self) -> T;

		fn top_left(&self) -> Point2D<T, S>;
		fn bottom_left(&self) -> Point2D<T, S>;
		fn top_right(&self) -> Point2D<T, S>;
		fn bottom_right(&self) -> Point2D<T, S>;

		fn with_size(self, size: Size2D<T, S>) -> Self;
		fn with_origin(self, origin: Point2D<T, S>) -> Self;

		fn inflate_top(self, t: T) -> Self;
		fn inflate_bottom(self, t: T) -> Self;
		fn inflate_left(self, t: T) -> Self;
		fn inflate_right(self, t: T) -> Self;

		fn deflate(self, width: T, height: T) -> Self;

		fn deflate_top(self, t: T) -> Self;
		fn deflate_bottom(self, t: T) -> Self;
		fn deflate_left(self, t: T) -> Self;
		fn deflate_right(self, t: T) -> Self;

		fn with_h(self, h: T) -> Self;
		fn with_w(self, w: T) -> Self;

		fn added_x(self, x: T) -> Self;
		fn added_y(self, y: T) -> Self;
		fn added_w(self, w: T) -> Self;
		fn added_h(self, h: T) -> Self;

		fn with_bottom(self, y: T) -> Self;
		fn with_top(self, y: T) -> Self;
		fn with_left(self, x: T) -> Self;
		fn with_right(self, x: T) -> Self;

		fn to_section_bounds(self) -> Section<'static>;
	}

	impl<T, S> BoundingBoxExt<T, S> for Rect<T, S>
	where
		T: Add<Output = T>
			+ AddAssign
			+ Sub<Output = T>
			+ SubAssign
			+ PartialOrd
			+ NumCast
			+ Copy,
	{
		#[inline(always)]
		fn top(&self) -> T {
			self.origin.y
		}
		#[inline(always)]
		fn bottom(&self) -> T {
			self.origin.y + self.size.height
		}
		#[inline(always)]
		fn left(&self) -> T {
			self.origin.x
		}
		#[inline(always)]
		fn right(&self) -> T {
			self.origin.x + self.size.width
		}

		#[inline(always)]
		fn top_left(&self) -> Point2D<T, S> {
			self.origin
		}
		#[inline(always)]
		fn bottom_left(&self) -> Point2D<T, S> {
			Point2D::new(self.origin.x, self.origin.y + self.size.height)
		}
		#[inline(always)]
		fn top_right(&self) -> Point2D<T, S> {
			Point2D::new(self.origin.x + self.size.width, self.origin.y)
		}
		#[inline(always)]
		fn bottom_right(&self) -> Point2D<T, S> {
			Point2D::new(
				self.origin.x + self.size.width,
				self.origin.y + self.size.height,
			)
		}

		#[inline(always)]
		fn with_size(mut self, size: Size2D<T, S>) -> Self {
			self.size = size;
			self
		}
		#[inline(always)]
		fn with_origin(mut self, origin: Point2D<T, S>) -> Self {
			self.origin = origin;
			self
		}

		#[inline(always)]
		fn inflate_top(mut self, t: T) -> Self {
			self.origin.y -= t;
			self.size.height += t;
			self
		}
		#[inline(always)]
		fn inflate_bottom(mut self, t: T) -> Self {
			self.size.height += t;
			self
		}
		#[inline(always)]
		fn inflate_left(mut self, t: T) -> Self {
			self.origin.x -= t;
			self.size.width += t;
			self
		}
		#[inline(always)]
		fn inflate_right(mut self, t: T) -> Self {
			self.size.width += t;
			self
		}

		#[inline]
		fn deflate(mut self, width: T, height: T) -> Self {
			self.origin.x += width;
			self.origin.y += height;
			self.size.width -= width;
			self.size.width -= width;
			self.size.height -= height;
			self.size.height -= height;

			self
		}

		#[inline(always)]
		fn deflate_top(mut self, t: T) -> Self {
			self.origin.y += t;
			self.size.height -= t;
			self
		}
		#[inline(always)]
		fn deflate_bottom(mut self, t: T) -> Self {
			self.size.height -= t;
			self
		}
		#[inline(always)]
		fn deflate_left(mut self, t: T) -> Self {
			self.origin.x += t;
			self.size.width -= t;
			self
		}
		#[inline(always)]
		fn deflate_right(mut self, t: T) -> Self {
			self.size.width -= t;
			self
		}

		#[inline(always)]
		fn with_h(mut self, h: T) -> Self {
			self.size.height = h;
			self
		}
		#[inline(always)]
		fn with_w(mut self, w: T) -> Self {
			self.size.width = w;
			self
		}

		#[inline(always)]
		fn added_x(mut self, x: T) -> Self {
			self.origin.x += x;
			self
		}
		#[inline(always)]
		fn added_y(mut self, y: T) -> Self {
			self.origin.y += y;
			self
		}
		#[inline(always)]
		fn added_w(mut self, w: T) -> Self {
			self.size.width += w;
			self
		}
		#[inline(always)]
		fn added_h(mut self, h: T) -> Self {
			self.size.height += h;
			self
		}

		#[inline(always)]
		fn with_bottom(mut self, bottom: T) -> Self {
			self.size.height = bottom - self.origin.y;
			self
		}
		#[inline]
		fn with_top(mut self, top: T) -> Self {
			if top > self.origin.y {
				self.size.height -= top - self.origin.y;
			} else {
				self.size.height += self.origin.y - top;
			}

			self.origin.y = top;
			self
		}
		#[inline]
		fn with_left(mut self, left: T) -> Self {
			if left > self.origin.x {
				self.size.width -= left - self.origin.x;
			} else {
				self.size.width += self.origin.x - left;
			}

			self.origin.x = left;
			self
		}
		#[inline(always)]
		fn with_right(mut self, right: T) -> Self {
			self.size.width = right - self.origin.x;
			self
		}

		#[inline]
		fn to_section_bounds(self) -> Section<'static> {
			let this = self.to_f32();
			Section {
				screen_position: this.origin.to_tuple(),
				bounds: this.size.to_tuple(),
				..Section::default()
			}
		}
	}
}

pub mod ext {

	use super::*;

	use crate::icons::RGBA8_UNORM_BPP;

	use winit::{
		dpi::{LogicalSize, PhysicalSize},
		window::Window,
	};

	pub trait ScreenPixelPointExt: Sized {
		fn to_norm(self, size: LogicalSize<u32>) -> ScreenNormPoint;
	}

	impl ScreenPixelPointExt for ScreenPixelPoint {
		#[inline]
		fn to_norm(self, window_size: LogicalSize<u32>) -> ScreenNormPoint {
			let window_size: LogicalSize<f32> = window_size.cast();
			let this: Point2D<f32, ScreenPixelSpace> = self.cast();

			ScreenNormPoint::new(
				(this.x / window_size.width - 0.5) * 2.0,
				(this.y / window_size.height - 0.5) * 2.0,
			)
		}
	}

	pub trait ScreenPhysicalPointExt: Sized {
		fn to_logical(self, scale_factor: f64) -> ScreenPixelPoint;
	}

	impl ScreenPhysicalPointExt for ScreenPhysicalPoint {
		#[inline]
		fn to_logical(self, scale_factor: f64) -> ScreenPixelPoint {
			let size = LogicalSize::from_physical(
				PhysicalSize::new(self.x, self.y),
				scale_factor,
			);
			ScreenPixelPoint::new(size.width, size.height)
		}
	}

	pub trait ScreenPixelSizeExt: Sized {
		fn of_window(window: &Window) -> Self;
		fn to_norm(self, size: LogicalSize<u32>) -> ScreenNormSize;
	}

	impl ScreenPixelSizeExt for ScreenPixelSize {
		#[inline]
		fn of_window(window: &Window) -> Self {
			let LogicalSize { width, height } =
				window.inner_size().to_logical(window.scale_factor());
			ScreenPixelSize::new(width, height)
		}
		#[inline]
		fn to_norm(self, window: LogicalSize<u32>) -> ScreenNormSize {
			let window: LogicalSize<f32> = window.cast();
			let this: Size2D<f32, ScreenPixelSpace> = self.cast();
			ScreenNormSize::new(
				(this.width / window.width) * 2.0,
				(this.height / window.height) * 2.0,
			)
		}
	}

	pub trait ScreenPhysicalSizeExt: Sized {
		fn to_logical(self, scale_factor: f64) -> ScreenPixelSize;
	}

	impl ScreenPhysicalSizeExt for ScreenPhysicalSize {
		#[inline]
		fn to_logical(self, scale_factor: f64) -> ScreenPixelSize {
			let size = LogicalSize::from_physical(
				PhysicalSize::new(self.width, self.height),
				scale_factor,
			);
			ScreenPixelSize::new(size.width, size.height)
		}
	}

	pub trait ScreenPixelRectExt: Sized {
		fn to_norm(self, size: LogicalSize<u32>) -> ScreenNormRect;
	}

	impl ScreenPixelRectExt for ScreenPixelRect {
		#[inline(always)]
		fn to_norm(self, size: LogicalSize<u32>) -> ScreenNormRect {
			ScreenNormRect::new(
				self.origin.to_norm(size),
				self.size.to_norm(size),
			)
		}
	}

	pub trait ScreenPhysicalRectExt: Sized {
		fn to_logical(self, scale_factor: f64) -> ScreenPixelRect;
	}

	impl ScreenPhysicalRectExt for ScreenPhysicalRect {
		#[inline(always)]
		fn to_logical(self, scale_factor: f64) -> ScreenPixelRect {
			ScreenPixelRect::new(
				self.origin.to_logical(scale_factor),
				self.size.to_logical(scale_factor),
			)
		}
	}

	pub trait TexPixelPointExt: Sized {
		fn to_norm(self, size: TexPixelSize) -> TexNormPoint;

		fn to_bytes(self) -> TexBytePoint;

		fn to_bytes_bpp(self, bpp: u32) -> TexBytePoint;
	}

	impl TexPixelPointExt for TexPixelPoint {
		#[inline]
		fn to_norm(self, size: TexPixelSize) -> TexNormPoint {
			let size: Size2D<f32, TexPixelSpace> = size.cast();
			let this: Point2D<f32, TexPixelSpace> = self.cast();
			TexNormPoint::new(this.x / size.width, this.y / size.height)
		}

		#[inline(always)]
		fn to_bytes(self) -> TexBytePoint {
			self.to_bytes_bpp(RGBA8_UNORM_BPP)
		}

		#[inline(always)]
		fn to_bytes_bpp(mut self, bpp: u32) -> TexBytePoint {
			self.x *= bpp;
			self.cast_unit()
		}
	}

	pub trait TexBytePointExt: Sized {
		fn to_pixels(self) -> TexPixelPoint;

		fn to_pixels_bpp(self, bpp: u32) -> TexPixelPoint;
	}

	impl TexBytePointExt for TexBytePoint {
		#[inline(always)]
		fn to_pixels(self) -> TexPixelPoint {
			self.to_pixels_bpp(RGBA8_UNORM_BPP)
		}

		#[inline(always)]
		fn to_pixels_bpp(mut self, bpp: u32) -> TexPixelPoint {
			self.x /= bpp;
			self.cast_unit()
		}
	}

	pub trait TexPixelSizeExt: Sized {
		fn to_norm(self, size: TexPixelSize) -> TexNormSize;

		fn to_bytes(self) -> TexByteSize;

		fn to_bytes_bpp(self, bpp: u32) -> TexByteSize;

		fn to_extent(self) -> wgpu::Extent3d;
	}

	impl TexPixelSizeExt for TexPixelSize {
		#[inline]
		fn to_norm(self, tex_size: TexPixelSize) -> TexNormSize {
			let tex_size: Size2D<f32, TexPixelSpace> = tex_size.cast();
			let this: Size2D<f32, TexPixelSpace> = self.cast();
			TexNormSize::new(
				this.width / tex_size.width,
				this.height / tex_size.height,
			)
		}

		#[inline(always)]
		fn to_bytes(self) -> TexByteSize {
			self.to_bytes_bpp(RGBA8_UNORM_BPP)
		}

		#[inline(always)]
		fn to_bytes_bpp(mut self, bpp: u32) -> TexByteSize {
			self.width *= bpp;
			self.cast_unit()
		}

		#[inline(always)]
		fn to_extent(self) -> wgpu::Extent3d {
			wgpu::Extent3d {
				width: self.width,
				height: self.height,
				depth: 1,
			}
		}
	}

	pub trait TexByteSizeExt: Sized {
		fn to_pixels(self) -> TexPixelSize;

		fn to_pixels_bpp(self, bpp: u32) -> TexPixelSize;
	}

	impl TexByteSizeExt for TexByteSize {
		#[inline(always)]
		fn to_pixels(self) -> TexPixelSize {
			self.to_pixels_bpp(RGBA8_UNORM_BPP)
		}

		#[inline(always)]
		fn to_pixels_bpp(mut self, bpp: u32) -> TexPixelSize {
			self.width /= bpp;
			self.cast_unit()
		}
	}

	pub trait TexPixelRectExt: Sized {
		fn to_norm(self, size: TexPixelSize) -> TexNormRect;

		fn to_bytes(self) -> TexByteRect;

		fn to_bytes_bpp(self, bpp: u32) -> TexByteRect;
	}

	impl TexPixelRectExt for TexPixelRect {
		#[inline(always)]
		fn to_norm(self, size: TexPixelSize) -> TexNormRect {
			TexNormRect::new(self.origin.to_norm(size), self.size.to_norm(size))
		}

		#[inline(always)]
		fn to_bytes(self) -> TexByteRect {
			TexByteRect::new(self.origin.to_bytes(), self.size.to_bytes())
		}

		#[inline(always)]
		fn to_bytes_bpp(self, bpp: u32) -> TexByteRect {
			TexByteRect::new(
				self.origin.to_bytes_bpp(bpp),
				self.size.to_bytes_bpp(bpp),
			)
		}
	}

	pub trait TexByteRectExt: Sized {
		fn to_pixels(self) -> TexPixelRect;

		fn to_pixels_bpp(self, bpp: u32) -> TexPixelRect;
	}

	impl TexByteRectExt for TexByteRect {
		#[inline(always)]
		fn to_pixels(self) -> TexPixelRect {
			TexPixelRect::new(self.origin.to_pixels(), self.size.to_pixels())
		}

		#[inline(always)]
		fn to_pixels_bpp(self, bpp: u32) -> TexPixelRect {
			TexPixelRect::new(
				self.origin.to_pixels_bpp(bpp),
				self.size.to_pixels_bpp(bpp),
			)
		}
	}
}

pub fn typed_scale_factor(
	scale_factor: f64,
) -> Scale<f64, ScreenPixelPoint, ScreenPhysicalPoint> {
	Scale::new(scale_factor)
}
