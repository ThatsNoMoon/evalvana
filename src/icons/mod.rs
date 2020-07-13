// Copyright 2020 Benjamin Scherer
// Licensed under the Open Software License version 3.0

mod logo;
mod ui;

use crate::geometry::{
	ext::*, TexBytePoint, TexByteSize, TexPixelPoint, TexPixelRect,
	TexPixelSize,
};

use std::convert::TryInto;

use image::{png::PngDecoder, ImageDecoder};
use winit::window::{Icon as WindowIcon, Theme};

pub const RGBA8_UNORM_BPP: u32 = 4;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconType {
	Close,
}

pub struct Icons {
	scale_factor: u8,
	theme: Theme,
	atlas: Vec<u8>,
	atlas_size: TexPixelSize,
	icons: Vec<TexPixelRect>,
}

impl Default for Icons {
	fn default() -> Icons {
		Icons::new(1.0, Theme::Dark)
	}
}

impl Icons {
	pub fn new(scale_factor: f64, theme: Theme) -> Icons {
		let scale_factor = norm_scale_factor(scale_factor);

		let mut this = Icons {
			scale_factor,
			theme,
			icons: vec![],
			atlas: vec![],
			atlas_size: TexPixelSize::zero(),
		};

		this.create_atlas();

		this
	}

	pub fn create_window_icon(&self) -> WindowIcon {
		icon_from_bytes(logo::for_scale_factor(self.scale_factor))
	}

	fn create_atlas(&mut self) {
		let mut icon_data =
			[ui::close::bytes_for(self.scale_factor, &self.theme)];

		icon_data.sort_unstable_by_key(|(_, size)| size.width);
		icon_data.sort_by_key(|(_, size)| size.height);

		let icon_data: Vec<_> = icon_data
			.iter()
			.copied()
			.map(|(icon_bytes, size)| {
				let icon_decoder = PngDecoder::new(icon_bytes).unwrap();
				let decoded_size = icon_decoder.dimensions();
				assert_eq!(decoded_size, size.to_tuple());
				let mut icon_pixels =
					vec![0; icon_decoder.total_bytes() as usize];
				icon_decoder.read_image(icon_pixels.as_mut_slice()).unwrap();
				(icon_pixels, size)
			})
			.collect();

		let mut icons = Vec::with_capacity(icon_data.len());

		let atlas_size = {
			let mut atlas_size = TexPixelSize::new(256 / RGBA8_UNORM_BPP, 64);

			'outer: loop {
				icons.clear();

				let mut iter = icon_data.iter();
				let &(_, icon_size) = iter.next().unwrap();
				if icon_size.height > atlas_size.height {
					atlas_size.height *= 2;
					continue 'outer;
				}

				if icon_size.width > atlas_size.width {
					atlas_size.width *= 2;
					continue 'outer;
				}

				let mut atlas_location = TexPixelPoint::zero();

				icons.push(TexPixelRect::new(atlas_location, icon_size));

				let mut last_height = icon_size.height;

				for &(_, icon_size) in iter {
					if icon_size.height > last_height {
						if icon_size.height
							> atlas_location.y + atlas_size.height
						{
							atlas_size.height *= 2;
							continue 'outer;
						}

						atlas_location.y += last_height;
						atlas_location.x = icon_size.width;
						last_height = icon_size.height;
					}

					if icon_size.width + atlas_location.x > atlas_size.width {
						atlas_size.width *= 2;
						continue 'outer;
					}

					icons.push(TexPixelRect::new(atlas_location, icon_size));

					atlas_location.x += icon_size.width;
				}

				break atlas_size;
			}
		};

		self.atlas = vec![0; atlas_size.to_bytes().area().try_into().unwrap()];

		for (icon_rect, icon_bytes) in icons
			.iter()
			.zip(icon_data.iter().map(|(icon_bytes, _)| icon_bytes))
		{
			copy_icon_bytes_to_atlas(
				icon_bytes.as_slice(),
				icon_rect.size.to_bytes(),
				self.atlas.as_mut_slice(),
				atlas_size.to_bytes(),
				icon_rect.origin.to_bytes(),
			);
		}

		self.atlas_size = atlas_size;
		self.icons = icons;
	}

	pub fn set_scale_factor(&mut self, scale_factor: f64) {
		self.scale_factor = norm_scale_factor(scale_factor);
		self.create_atlas();
	}

	pub fn set_theme(&mut self, theme: Theme) {
		self.theme = theme;
		self.create_atlas();
	}

	pub fn texture_atlas_size(&self) -> TexPixelSize {
		self.atlas_size
	}

	pub fn fill_texture_atlas(
		&self,
		device: &wgpu::Device,
		texture: &wgpu::Texture,
		encoder: &mut wgpu::CommandEncoder,
	) {
		let byte_size = self.atlas_size.to_bytes();

		let temp_buf = device.create_buffer_with_data(
			self.atlas.as_slice(),
			wgpu::BufferUsage::COPY_SRC,
		);

		encoder.copy_buffer_to_texture(
			wgpu::BufferCopyView {
				buffer: &temp_buf,
				offset: 0,
				bytes_per_row: byte_size.width,
				rows_per_image: byte_size.height,
			},
			wgpu::TextureCopyView {
				texture,
				mip_level: 0,
				array_layer: 0,
				origin: wgpu::Origin3d::ZERO,
			},
			self.atlas_size.to_extent(),
		);
	}

	pub fn get_icon_descriptor(&self, icon: IconType) -> TexPixelRect {
		self.icons[icon as u32 as usize]
	}
}

#[cfg(target_os = "windows")]
impl Icons {
	pub fn create_taskbar_icon(&self) -> WindowIcon {
		icon_from_bytes(logo::for_scale_factor(self.scale_factor * 2))
	}
}

fn icon_from_bytes(icon_bytes: &[u8]) -> WindowIcon {
	let logo_decoder = PngDecoder::new(icon_bytes).unwrap();
	let (logo_w, logo_h) = logo_decoder.dimensions();
	let mut logo_pixels = vec![0; logo_decoder.total_bytes() as usize];
	logo_decoder.read_image(logo_pixels.as_mut_slice()).unwrap();

	WindowIcon::from_rgba(logo_pixels, logo_w, logo_h).unwrap()
}

fn norm_scale_factor(scale_factor: f64) -> u8 {
	(scale_factor.ceil() as u8)
		.checked_next_power_of_two()
		.unwrap_or_else(|| panic!("Invalid scale factor {}", scale_factor))
}

fn copy_icon_bytes_to_atlas(
	icon_bytes: &[u8],
	icon_size: TexByteSize,
	atlas: &mut [u8],
	atlas_size: TexByteSize,
	atlas_location: TexBytePoint,
) {
	let icon_width = icon_size.width.try_into().unwrap();
	let icon_height = icon_size.height.try_into().unwrap();
	let atlas_width = atlas_size.width.try_into().unwrap();
	let atlas_x = atlas_location.x.try_into().unwrap();
	let atlas_y = atlas_location.y.try_into().unwrap();

	atlas
		.chunks_exact_mut(atlas_width)
		.skip(atlas_y)
		.take(icon_height)
		.map(|chunk| &mut chunk[atlas_x..atlas_x + icon_width])
		.zip(icon_bytes.chunks_exact(icon_width))
		.for_each(|(dest, source)| dest.copy_from_slice(source));
}
