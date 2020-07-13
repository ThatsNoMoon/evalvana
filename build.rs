// Copyright 2020 Benjamin Scherer
// Licensed under the Open Software License version 3.0

use std::fs;

use shaderc::{CompileOptions, Compiler, OptimizationLevel, ShaderKind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
	fs::create_dir_all("target/shaders/")?;

	let mut compiler = Compiler::new().unwrap();

	let options = match std::env::var("PROFILE").unwrap().as_str() {
		"release" => {
			let mut options = CompileOptions::new().unwrap();
			options.set_optimization_level(OptimizationLevel::Performance);
			Some(options)
		}
		"debug" => {
			let mut options = CompileOptions::new().unwrap();
			options.set_optimization_level(OptimizationLevel::Zero);
			Some(options)
		}
		_ => None,
	};

	for entry in fs::read_dir("assets/shaders")? {
		let entry = entry?;

		if entry.file_type()?.is_file() {
			let input_path = entry.path();

			let input_name: String =
				entry.file_name().to_string_lossy().into_owned();

			let shader_kind = input_path
				.extension()
				.and_then(|ext| match ext.to_string_lossy().as_ref() {
					"vert" => Some(ShaderKind::Vertex),
					"frag" => Some(ShaderKind::Fragment),
					_ => None,
				})
				.ok_or_else(|| {
					format!(
							"Could not identify shader type of {:?}; expected .vert or .frag extension",
							input_path
						)
				})?;

			println!("cargo:rerun-if-changed={:?}", input_path);

			let source = fs::read_to_string(&input_path)?;
			let artifact = compiler.compile_into_spirv(
				&source,
				shader_kind,
				&input_name,
				"main",
				options.as_ref(),
			)?;

			if artifact.get_num_warnings() > 0 {
				eprintln!(
					"Shader compilation warning: {}: {}",
					input_name,
					artifact.get_warning_messages()
				);
			}

			let output_path = format!("target/shaders/{}.spv", input_name);

			fs::write(&output_path, artifact.as_binary_u8())?;
		}
	}

	Ok(())
}
