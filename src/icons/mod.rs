mod logo;
mod ui;

use crate::interface::Theme;

use std::convert::TryInto;

use image::{error::ImageResult, png::PngDecoder, ImageDecoder};
use winit::window::Icon as WindowIcon;

pub const BPP: u32 = 4;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconType {
	Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IconDescriptor {
	pub size: (u32, u32),
	pub atlas_location: (u32, u32),
}

pub struct Icons {
	scale_factor: u8,
	theme: Theme,
	atlas: Vec<u8>,
	atlas_size: (u32, u32),
	icons: Vec<IconDescriptor>,
}

impl Default for Icons {
	fn default() -> Icons {
		Icons::new(1.0, Theme::default())
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
			atlas_size: (0, 0),
		};

		this.create_atlas();

		this
	}

	pub fn create_window_icon(&self) -> WindowIcon {
		icon_from_bytes(logo::for_scale_factor(self.scale_factor))
	}

	fn create_atlas(&mut self) {
		let mut icon_data =
			[ui::close::bytes_for(self.scale_factor, self.theme)];

		icon_data.sort_unstable_by_key(|(_, (width, _))| *width);
		icon_data.sort_by_key(|(_, (_, height))| *height);

		let icon_data: Vec<_> = icon_data
			.iter()
			.copied()
			.map(|(icon_bytes, size)| {
				let icon_decoder = PngDecoder::new(icon_bytes).unwrap();
				let decoded_size = icon_decoder.dimensions();
				assert_eq!(decoded_size, size);
				let mut icon_pixels =
					vec![0; icon_decoder.total_bytes() as usize];
				icon_decoder.read_image(icon_pixels.as_mut_slice()).unwrap();
				(icon_pixels, size)
			})
			.collect();

		let mut icons = Vec::with_capacity(icon_data.len());

		let (atlas_width, atlas_height) = {
			let (mut atlas_width, mut atlas_height): (u32, u32) =
				(256 / BPP, 64);

			'outer: loop {
				icons.clear();

				let mut iter = icon_data.iter();
				let &(_, (width, height)) = iter.next().unwrap();
				if height > atlas_height {
					atlas_height *= 2;
					continue 'outer;
				}

				if width > atlas_width {
					atlas_width *= 2;
					continue 'outer;
				}

				let (mut atlas_x, mut atlas_y) = (0, 0);

				icons.push(IconDescriptor {
					size: (width, height),
					atlas_location: (atlas_x, atlas_y),
				});

				let mut last_height = height;

				for &(_, (width, height)) in iter {
					if height > last_height {
						if height > atlas_y + atlas_height {
							atlas_height *= 2;
							continue 'outer;
						}

						atlas_y += last_height;
						atlas_x = width;
						last_height = height;
					}

					if width + atlas_x > atlas_width {
						atlas_width *= 2;
						continue 'outer;
					}

					icons.push(IconDescriptor {
						size: (width, height),
						atlas_location: (atlas_x, atlas_y),
					});

					atlas_x += width;
				}

				break (atlas_width, atlas_height);
			}
		};

		self.atlas =
			vec![0; (BPP * atlas_width * atlas_height).try_into().unwrap()];

		for (icon_descriptor, icon_bytes) in icons
			.iter()
			.zip(icon_data.iter().map(|(icon_bytes, _)| icon_bytes))
		{
			let &IconDescriptor {
				size: (width, height),
				atlas_location: (atlas_x, atlas_y),
			} = icon_descriptor;

			copy_icon_bytes_to_atlas(
				icon_bytes.as_slice(),
				(BPP * width, height),
				self.atlas.as_mut_slice(),
				BPP * atlas_width,
				(BPP * atlas_x, atlas_y),
			);
		}

		self.atlas_size = (atlas_width, atlas_height);
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

	pub fn texture_atlas_size(&self) -> (u32, u32) {
		self.atlas_size
	}

	pub fn fill_texture_atlas(
		&self,
		device: &wgpu::Device,
		texture: &wgpu::Texture,
		encoder: &mut wgpu::CommandEncoder,
	) {
		let (width, height) = self.atlas_size;

		let temp_buf = device
			.create_buffer_mapped(
				(BPP * width * height).try_into().unwrap(),
				wgpu::BufferUsage::COPY_SRC,
			)
			.fill_from_slice(self.atlas.as_slice());

		encoder.copy_buffer_to_texture(
			wgpu::BufferCopyView {
				buffer: &temp_buf,
				offset: 0,
				row_pitch: BPP * width,
				image_height: height,
			},
			wgpu::TextureCopyView {
				texture,
				mip_level: 0,
				array_layer: 0,
				origin: wgpu::Origin3d::ZERO,
			},
			wgpu::Extent3d {
				width,
				height,
				depth: 1,
			},
		);
	}

	pub fn get_icon_descriptor(&self, icon: IconType) -> IconDescriptor {
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
	(icon_width, icon_height): (u32, u32),
	atlas: &mut [u8],
	atlas_width: u32,
	(atlas_x, atlas_y): (u32, u32),
) {
	let icon_width = icon_width.try_into().unwrap();
	let icon_height = icon_height.try_into().unwrap();
	let atlas_width = atlas_width.try_into().unwrap();
	let atlas_x = atlas_x.try_into().unwrap();
	let atlas_y = atlas_y.try_into().unwrap();

	atlas
		.chunks_exact_mut(atlas_width)
		.skip(atlas_y)
		.take(icon_height)
		.map(|chunk| &mut chunk[atlas_x..atlas_x + icon_width])
		.zip(icon_bytes.chunks_exact(icon_width))
		.for_each(|(dest, source)| dest.copy_from_slice(source));
}
