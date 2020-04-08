use super::{
	ColorVertex, TextureVertex, VertexIndex, VertexIndexFormat, COLOR_FORMAT,
};

use crate::geometry::{ext::TexPixelSizeExt, TexPixelSize};

const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

#[derive(Debug)]
pub struct Pipelines {
	color_pipeline: wgpu::RenderPipeline,
	color_bind_group: wgpu::BindGroup,
	texture_pipeline: wgpu::RenderPipeline,
	texture_bind_group: wgpu::BindGroup,
	texture_atlas: wgpu::Texture,
}

impl Pipelines {
	pub fn new(device: &wgpu::Device, tex_size: TexPixelSize) -> Pipelines {
		let (color_pipeline, color_bind_group) = create_color_pipeline(device);
		let (texture_pipeline, texture_bind_group, texture_atlas) =
			create_texture_pipeline(device, tex_size);

		Pipelines {
			color_pipeline,
			color_bind_group,
			texture_pipeline,
			texture_bind_group,
			texture_atlas,
		}
	}

	pub fn color_pipeline(&self) -> &wgpu::RenderPipeline {
		&self.color_pipeline
	}

	pub fn texture_pipeline(&self) -> &wgpu::RenderPipeline {
		&self.texture_pipeline
	}

	pub fn color_bind_group(&self) -> &wgpu::BindGroup {
		&self.color_bind_group
	}

	pub fn texture_bind_group(&self) -> &wgpu::BindGroup {
		&self.texture_bind_group
	}

	pub fn texture_atlas(&self) -> &wgpu::Texture {
		&self.texture_atlas
	}
}

fn create_color_pipeline(
	device: &wgpu::Device,
) -> (wgpu::RenderPipeline, wgpu::BindGroup) {
	let vs = include_bytes!(concat!(
		env!("CARGO_MANIFEST_DIR"),
		"/target/shaders/color_shader.vert.spv"
	));
	let vs_module = device.create_shader_module(
		&wgpu::read_spirv(std::io::Cursor::new(&vs[..])).unwrap(),
	);

	let fs = include_bytes!(concat!(
		env!("CARGO_MANIFEST_DIR"),
		"/target/shaders/color_shader.frag.spv"
	));
	let fs_module = device.create_shader_module(
		&wgpu::read_spirv(std::io::Cursor::new(&fs[..])).unwrap(),
	);

	let bind_group_layout =
		device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			bindings: &[],
		});
	let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
		layout: &bind_group_layout,
		bindings: &[],
	});
	let pipeline_layout =
		device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			bind_group_layouts: &[&bind_group_layout],
		});

	let render_pipeline =
		device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			layout: &pipeline_layout,
			vertex_stage: wgpu::ProgrammableStageDescriptor {
				module: &vs_module,
				entry_point: "main",
			},
			fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
				module: &fs_module,
				entry_point: "main",
			}),
			rasterization_state: Some(wgpu::RasterizationStateDescriptor {
				front_face: wgpu::FrontFace::Ccw,
				cull_mode: wgpu::CullMode::None,
				depth_bias: 0,
				depth_bias_slope_scale: 0.0,
				depth_bias_clamp: 0.0,
			}),
			primitive_topology: wgpu::PrimitiveTopology::TriangleList,
			color_states: &[wgpu::ColorStateDescriptor {
				format: COLOR_FORMAT,
				color_blend: wgpu::BlendDescriptor {
					src_factor: wgpu::BlendFactor::SrcAlpha,
					dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
					operation: wgpu::BlendOperation::Add,
				},
				alpha_blend: wgpu::BlendDescriptor {
					src_factor: wgpu::BlendFactor::One,
					dst_factor: wgpu::BlendFactor::Zero,
					operation: wgpu::BlendOperation::Add,
				},
				write_mask: wgpu::ColorWrite::ALL,
			}],
			depth_stencil_state: None,
			index_format: VertexIndex::WGPU_FORMAT,
			vertex_buffers: &[wgpu::VertexBufferDescriptor {
				stride: std::mem::size_of::<ColorVertex>()
					as wgpu::BufferAddress,
				step_mode: wgpu::InputStepMode::Vertex,
				attributes: &[
					wgpu::VertexAttributeDescriptor {
						offset: memoffset::offset_of!(ColorVertex, pos)
							as wgpu::BufferAddress,
						format: wgpu::VertexFormat::Float2,
						shader_location: 0,
					},
					wgpu::VertexAttributeDescriptor {
						offset: memoffset::offset_of!(ColorVertex, color)
							as wgpu::BufferAddress,
						format: wgpu::VertexFormat::Float3,
						shader_location: 1,
					},
				],
			}],
			sample_count: 1,
			sample_mask: !0,
			alpha_to_coverage_enabled: false,
		});

	(render_pipeline, bind_group)
}

fn create_texture_pipeline(
	device: &wgpu::Device,
	tex_size: TexPixelSize,
) -> (wgpu::RenderPipeline, wgpu::BindGroup, wgpu::Texture) {
	let vs = include_bytes!(concat!(
		env!("CARGO_MANIFEST_DIR"),
		"/target/shaders/texture_shader.vert.spv"
	));
	let vs_module = device.create_shader_module(
		&wgpu::read_spirv(std::io::Cursor::new(&vs[..])).unwrap(),
	);

	let fs = include_bytes!(concat!(
		env!("CARGO_MANIFEST_DIR"),
		"/target/shaders/texture_shader.frag.spv"
	));
	let fs_module = device.create_shader_module(
		&wgpu::read_spirv(std::io::Cursor::new(&fs[..])).unwrap(),
	);

	let bind_group_layout =
		device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			bindings: &[
				wgpu::BindGroupLayoutBinding {
					binding: 1,
					visibility: wgpu::ShaderStage::FRAGMENT,
					ty: wgpu::BindingType::SampledTexture {
						multisampled: false,
						dimension: wgpu::TextureViewDimension::D2,
					},
				},
				wgpu::BindGroupLayoutBinding {
					binding: 2,
					visibility: wgpu::ShaderStage::FRAGMENT,
					ty: wgpu::BindingType::Sampler,
				},
			],
		});

	let texture = device.create_texture(&wgpu::TextureDescriptor {
		size: tex_size.to_extent(),
		sample_count: 1,
		array_layer_count: 1,
		mip_level_count: 1,
		dimension: wgpu::TextureDimension::D2,
		format: TEXTURE_FORMAT,
		usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
	});

	let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
		address_mode_u: wgpu::AddressMode::ClampToEdge,
		address_mode_v: wgpu::AddressMode::ClampToEdge,
		address_mode_w: wgpu::AddressMode::ClampToEdge,
		mag_filter: wgpu::FilterMode::Linear,
		min_filter: wgpu::FilterMode::Linear,
		mipmap_filter: wgpu::FilterMode::Linear,
		lod_min_clamp: 0.0,
		lod_max_clamp: 0.0,
		compare_function: wgpu::CompareFunction::Always,
	});

	let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
		layout: &bind_group_layout,
		bindings: &[
			wgpu::Binding {
				binding: 1,
				resource: wgpu::BindingResource::TextureView(
					&texture.create_default_view(),
				),
			},
			wgpu::Binding {
				binding: 2,
				resource: wgpu::BindingResource::Sampler(&sampler),
			},
		],
	});

	let pipeline_layout =
		device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			bind_group_layouts: &[&bind_group_layout],
		});

	let render_pipeline =
		device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			layout: &pipeline_layout,
			vertex_stage: wgpu::ProgrammableStageDescriptor {
				module: &vs_module,
				entry_point: "main",
			},
			fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
				module: &fs_module,
				entry_point: "main",
			}),
			rasterization_state: Some(wgpu::RasterizationStateDescriptor {
				front_face: wgpu::FrontFace::Ccw,
				cull_mode: wgpu::CullMode::None,
				depth_bias: 0,
				depth_bias_slope_scale: 0.0,
				depth_bias_clamp: 0.0,
			}),
			primitive_topology: wgpu::PrimitiveTopology::TriangleList,
			color_states: &[wgpu::ColorStateDescriptor {
				format: COLOR_FORMAT,
				color_blend: wgpu::BlendDescriptor {
					src_factor: wgpu::BlendFactor::SrcAlpha,
					dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
					operation: wgpu::BlendOperation::Add,
				},
				alpha_blend: wgpu::BlendDescriptor {
					src_factor: wgpu::BlendFactor::One,
					dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
					operation: wgpu::BlendOperation::Add,
				},
				write_mask: wgpu::ColorWrite::ALL,
			}],
			depth_stencil_state: None,
			index_format: VertexIndex::WGPU_FORMAT,
			vertex_buffers: &[wgpu::VertexBufferDescriptor {
				stride: std::mem::size_of::<TextureVertex>()
					as wgpu::BufferAddress,
				step_mode: wgpu::InputStepMode::Vertex,
				attributes: &[
					wgpu::VertexAttributeDescriptor {
						offset: memoffset::offset_of!(TextureVertex, pos)
							as wgpu::BufferAddress,
						format: wgpu::VertexFormat::Float2,
						shader_location: 0,
					},
					wgpu::VertexAttributeDescriptor {
						offset: memoffset::offset_of!(TextureVertex, tex_coord)
							as wgpu::BufferAddress,
						format: wgpu::VertexFormat::Float2,
						shader_location: 1,
					},
				],
			}],
			sample_count: 1,
			sample_mask: !0,
			alpha_to_coverage_enabled: false,
		});

	(render_pipeline, bind_group, texture)
}
